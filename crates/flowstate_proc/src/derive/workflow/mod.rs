mod attr;

use quote::quote;
use syn::{DeriveInput, Fields, FieldsNamed, Ident, LifetimeParam, Type, TypeParam};

use crate::{
    derive::workflow::attr::{FlowstateAttrArgs, StateAttrArgs},
    err::{
        DuplicateStateAttribute, ExtraGenericLifetimeParameter, InvalidStateFieldType,
        MissingFlowstateAttributeOnWorkflow, MissingGenericTypeParameterForState,
        MissingStateAttribute, UnexpectedUnnamedField, UnsupportedAdditionalGenericTypeParameters,
        UnsupportedBoundsOnGenericLifetimeParameterForWorkflow,
        UnsupportedBoundsOnGenericTypeParameterForState, UnsupportedEnumOrUnion,
        UnsupportedGenericConstParameter, UnsupportedGenericWhereClause, UnsupportedTupleStruct,
        UnsupportedUnitStruct,
    },
};

pub fn derive_workflow_impl(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let s = validate_input(&input)?;

    generate_impls(&s)
}

struct ValidatedWorkflowStruct<'s> {
    ident: &'s Ident,
    args: FlowstateAttrArgs,
    generics: ValidatedStructGenerics<'s>,
    fields: ValidatedStructFields<'s>,
}

fn validate_input(input: &DeriveInput) -> syn::Result<ValidatedWorkflowStruct<'_>> {
    let args = validate_flowstate_attr(input)?;
    let generics = validate_generics(input)?;
    let fields = validate_struct(input, &generics)?;

    Ok(ValidatedWorkflowStruct {
        ident: &input.ident,
        args,
        generics,
        fields,
    })
}

fn validate_flowstate_attr(input: &DeriveInput) -> syn::Result<FlowstateAttrArgs> {
    let Some(attr) = input.attrs.iter().find(|a| a.path().is_ident("flowstate")) else {
        return Err(MissingFlowstateAttributeOnWorkflow::at(&input.ident).into());
    };

    attr.parse_args()
}

struct ValidatedStructGenerics<'s> {
    workflow_lifetime_param: Option<&'s LifetimeParam>,
    state_type_param: &'s TypeParam,
}

fn validate_generics(input: &DeriveInput) -> syn::Result<ValidatedStructGenerics<'_>> {
    let workflow_lifetime_param = validate_workflow_lifetime_generic(input)?;
    validate_generic_const_params(input)?;
    let state_type_param = validate_state_type_generic(input)?;
    validate_where_clause(input)?;

    Ok(ValidatedStructGenerics {
        workflow_lifetime_param,
        state_type_param,
    })
}

fn validate_workflow_lifetime_generic(
    input: &DeriveInput,
) -> syn::Result<Option<&'_ LifetimeParam>> {
    let mut lifetime_params = input.generics.lifetimes();

    let Some(workflow_lifetime) = lifetime_params.next() else {
        return Ok(None);
    };

    if let Some(extra_lifetime_param) = lifetime_params.next() {
        return Err(ExtraGenericLifetimeParameter::at(extra_lifetime_param).into());
    }

    if let Some(bounds) = workflow_lifetime.bounds.first() {
        return Err(UnsupportedBoundsOnGenericLifetimeParameterForWorkflow::at(bounds).into());
    }

    Ok(Some(workflow_lifetime))
}

fn validate_generic_const_params(input: &DeriveInput) -> syn::Result<()> {
    if let Some(const_param) = input.generics.const_params().next() {
        return Err(UnsupportedGenericConstParameter::at(const_param).into());
    }

    Ok(())
}

fn validate_state_type_generic(input: &DeriveInput) -> syn::Result<&'_ TypeParam> {
    let mut type_params = input.generics.type_params();

    let Some(state_type_param) = type_params.next() else {
        return Err(MissingGenericTypeParameterForState::at(&input.ident).into());
    };

    if let Some(extra_type_param) = type_params.next() {
        return Err(UnsupportedAdditionalGenericTypeParameters::at(&extra_type_param).into());
    }

    if let Some(bound) = state_type_param.bounds.first() {
        return Err(UnsupportedBoundsOnGenericTypeParameterForState::at(bound).into());
    }

    Ok(state_type_param)
}

fn validate_where_clause(input: &DeriveInput) -> syn::Result<()> {
    if let Some(where_clause) = &input.generics.where_clause {
        return Err(UnsupportedGenericWhereClause::at(where_clause).into());
    }

    Ok(())
}

fn validate_struct<'input>(
    input: &'input DeriveInput,
    validated_generics: &ValidatedStructGenerics<'_>,
) -> syn::Result<ValidatedStructFields<'input>> {
    let data = match &input.data {
        syn::Data::Struct(data) => data,
        syn::Data::Enum(_) | syn::Data::Union(_) => {
            return Err(UnsupportedEnumOrUnion::at(&input.ident).into());
        }
    };

    match &data.fields {
        Fields::Named(fields) => validate_struct_fields(fields, validated_generics),
        Fields::Unnamed(_) => Err(UnsupportedTupleStruct::at(&data.fields).into()),
        Fields::Unit => Err(UnsupportedUnitStruct::at(&data.fields).into()),
    }
}

struct Field<'a> {
    ident: &'a Ident,
    ty: &'a Type,
}

struct ValidatedStructFields<'a> {
    state: Field<'a>,
    rest: Vec<Field<'a>>,
}

fn validate_struct_fields<'a>(
    fields: &'a FieldsNamed,
    validated_generics: &ValidatedStructGenerics<'_>,
) -> syn::Result<ValidatedStructFields<'a>> {
    let mut state = None;
    let mut rest = Vec::new();

    for field in &fields.named {
        let Some(ident) = &field.ident else {
            return Err(UnexpectedUnnamedField::at(field).into());
        };

        if let Some(state_attr) = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("state"))
        {
            match state_attr.meta {
                syn::Meta::Path(_) => {}
                syn::Meta::List(_) | syn::Meta::NameValue(_) => {
                    let _: StateAttrArgs = state_attr.parse_args()?;
                }
            }

            if state.is_some() {
                return Err(DuplicateStateAttribute::at(state_attr).into());
            }

            let Type::Path(type_path) = &field.ty else {
                return Err(InvalidStateFieldType::at(&field.ty)
                    .with(
                        ident.to_string(),
                        validated_generics.state_type_param.ident.to_string(),
                    )
                    .into());
            };

            let Some(state_ident) = type_path.path.get_ident() else {
                return Err(InvalidStateFieldType::at(type_path)
                    .with(
                        ident.to_string(),
                        validated_generics.state_type_param.ident.to_string(),
                    )
                    .into());
            };

            if *state_ident != validated_generics.state_type_param.ident {
                return Err(InvalidStateFieldType::at(type_path)
                    .with(
                        ident.to_string(),
                        validated_generics.state_type_param.ident.to_string(),
                    )
                    .into());
            }

            state = Some(Field {
                ident,
                ty: &field.ty,
            })
        } else {
            rest.push(Field {
                ident,
                ty: &field.ty,
            })
        }
    }

    let Some(state) = state else {
        return Err(MissingStateAttribute::at(fields).into());
    };

    Ok(ValidatedStructFields { state, rest })
}

fn generate_impls(s: &ValidatedWorkflowStruct) -> syn::Result<proc_macro2::TokenStream> {
    let constructor_impl = generate_constructor_impl(s);
    let transition_helpers = generate_transition_helpers(s);
    let workflow_impl = generate_workflow_impl(s);
    let state_trait = generate_state_trait(s);

    Ok(quote! {
        #constructor_impl
        #transition_helpers
        #workflow_impl
        #state_trait
    })
}

fn generate_constructor_impl(s: &ValidatedWorkflowStruct) -> proc_macro2::TokenStream {
    let ident = s.ident;
    let state_generic_ident = &s.generics.state_type_param.ident;
    let workflow_generics = match s.generics.workflow_lifetime_param {
        Some(workflow_lifetime_param) => {
            let workflow_lifetime = &workflow_lifetime_param.lifetime;
            quote! {
                #workflow_lifetime, #state_generic_ident
            }
        }
        None => quote! { #state_generic_ident },
    };
    let state_field_ident = s.fields.state.ident;
    let rest_field_init_params = s.fields.rest.iter().map(|Field { ident, ty }| {
        quote! {
            #ident: #ty
        }
    });
    let rest_field_idents = s.fields.rest.iter().map(|Field { ident, .. }| {
        quote! {
            #ident
        }
    });

    quote! {
        impl<#workflow_generics> #ident<#workflow_generics> {
            fn new(#state_field_ident: #state_generic_ident, #(#rest_field_init_params,)*) -> Self {
                Self {
                    #state_field_ident,
                    #(#rest_field_idents,)*
                }
            }
        }
    }
}

fn generate_transition_helpers(s: &ValidatedWorkflowStruct) -> proc_macro2::TokenStream {
    let ident = s.ident;
    let state_generic_ident = &s.generics.state_type_param.ident;
    let workflow_lifetime = match &s.generics.workflow_lifetime_param {
        Some(LifetimeParam { lifetime, .. }) => quote! { #lifetime },
        None => quote! { 'static },
    };
    let workflow_generics = match s.generics.workflow_lifetime_param {
        Some(workflow_lifetime_param) => {
            let workflow_lifetime = &workflow_lifetime_param.lifetime;
            quote! {
                #workflow_lifetime, #state_generic_ident
            }
        }
        None => quote! { #state_generic_ident },
    };
    let next_state_ident = quote! { NextState };
    let next_state_workflow_generics = match s.generics.workflow_lifetime_param {
        Some(workflow_lifetime_param) => {
            let workflow_lifetime = &workflow_lifetime_param.lifetime;
            quote! {
                #workflow_lifetime, #next_state_ident
            }
        }
        None => quote! { #next_state_ident },
    };
    let state_field_ident = &s.fields.state.ident;
    let result_type = &s.args.result_type;
    let rest_field_transition_assignments: Vec<_> = s
        .fields
        .rest
        .iter()
        .map(|Field { ident, .. }| {
            quote! {
                #ident: self.#ident
            }
        })
        .collect();
    let workflow_state_trait = if s.args.is_async {
        quote! { ::flowstate::AsyncWorkflowState }
    } else {
        quote! { ::flowstate::WorkflowState }
    };
    let transition_type = if s.args.is_async {
        quote! { ::flowstate::AsyncTransition }
    } else {
        quote! { ::flowstate::Transition }
    };

    quote! {
        impl<#workflow_generics> #ident<#workflow_generics>
        {
            fn transition<#next_state_ident>(
                self,
                next_state: #next_state_ident,
            ) -> #transition_type<#workflow_lifetime, #result_type>
            where
                #ident<#next_state_workflow_generics>: #workflow_state_trait<#workflow_lifetime, #result_type> + #workflow_lifetime,
            {
                ::std::ops::ControlFlow::Continue(Box::new(#ident {
                    #state_field_ident: next_state,
                    #(#rest_field_transition_assignments,)*
                }))
            }

            fn transition_with<#next_state_ident, Fn>(
                self,
                map_fn: Fn,
            ) -> #transition_type<#workflow_lifetime, #result_type>
            where
                #ident<#next_state_workflow_generics>: #workflow_state_trait<#workflow_lifetime, #result_type> + #workflow_lifetime,
                Fn: FnOnce(#state_generic_ident) -> #next_state_ident,
            {
                ::std::ops::ControlFlow::Continue(Box::new(#ident {
                    #state_field_ident: map_fn(self.#state_field_ident),
                    #(#rest_field_transition_assignments,)*
                }))
            }
        }
    }
}

fn generate_workflow_impl(s: &ValidatedWorkflowStruct) -> proc_macro2::TokenStream {
    let ident = s.ident;
    let state_generic_ident = &s.generics.state_type_param.ident;
    let state_field_ident = &s.fields.state.ident;
    let workflow_generics = match s.generics.workflow_lifetime_param {
        Some(workflow_lifetime_param) => {
            let workflow_lifetime = &workflow_lifetime_param.lifetime;
            quote! {
                #workflow_lifetime, #state_generic_ident
            }
        }
        None => quote! { #state_generic_ident },
    };

    quote! {
        impl<#workflow_generics> ::flowstate::Workflow for #ident<#workflow_generics>
        where
            #state_generic_ident: ::flowstate::State,
        {
            fn state(&self) -> &dyn ::flowstate::State {
                &self.#state_field_ident
            }
        }
    }
}

fn generate_state_trait(s: &ValidatedWorkflowStruct) -> Option<proc_macro2::TokenStream> {
    let ident = s.ident;
    let state_generic_ident = &s.generics.state_type_param.ident;
    let result_type = &s.args.result_type;
    let Some(state_trait_ident) = &s.args.state_trait_ident else {
        return None;
    };
    let workflow_lifetime = match &s.generics.workflow_lifetime_param {
        Some(LifetimeParam { lifetime, .. }) => quote! { #lifetime },
        None => quote! { 'static },
    };
    let workflow_generics = match s.generics.workflow_lifetime_param {
        Some(workflow_lifetime_param) => {
            let workflow_lifetime = &workflow_lifetime_param.lifetime;
            quote! {
                #workflow_lifetime, #state_generic_ident
            }
        }
        None => quote! { #state_generic_ident },
    };
    let trait_generics = s
        .generics
        .workflow_lifetime_param
        .as_ref()
        .map(|LifetimeParam { lifetime, .. }| quote! { #lifetime });
    let trait_generics_bracketed = trait_generics
        .as_ref()
        .map(|trait_generics| quote! { <#trait_generics> });
    let workflow_state_trait = if s.args.is_async {
        quote! { ::flowstate::AsyncWorkflowState }
    } else {
        quote! { ::flowstate::WorkflowState }
    };
    let transition_type = if s.args.is_async {
        quote! { ::flowstate::AsyncTransition }
    } else {
        quote! { ::flowstate::Transition }
    };
    let state_trait_attrs = s.args.is_async.then(|| {
        quote! {
            #[::flowstate::async_state]
        }
    });
    let state_generic_bounds = if s.args.is_async {
        quote! { ::flowstate::State + Send }
    } else {
        quote! { ::flowstate::State }
    };
    let async_modifier = s.args.is_async.then(|| quote! { async });
    let await_operator = s.args.is_async.then(|| quote! { .await });

    Some(quote! {
        /// Each implementation represents the workflow in a specific state and
        /// defines the transition logic.
        ///
        /// # `next`
        ///
        /// Consumes the current workflow state and returns either:
        ///
        /// 1. The next workflow state
        /// 2. The workflow result
        ///
        /// Return [`self.transition(next_state)`] to transition to the
        /// workflow to the next state.
        ///
        /// Return [`self.finish(result)`](flowstate::WorkflowState::finish) or
        /// [`self.finish_with(|workflow| result)`](flowstate::WorkflowState::finish_with)
        /// to terminate the workflow with a result.
        ///
        #state_trait_attrs
        trait #state_trait_ident #trait_generics_bracketed: ::flowstate::Workflow {
            fn state_name(&self) -> String {
                self.state().name()
            }

            #async_modifier fn next(self: Box<Self>) -> #transition_type<#workflow_lifetime, #result_type>;
        }

        #state_trait_attrs
        impl<#workflow_generics> #workflow_state_trait<#workflow_lifetime, #result_type> for #ident<#workflow_generics>
        where
            #state_generic_ident: #state_generic_bounds,
            #ident<#workflow_generics>: #state_trait_ident #trait_generics_bracketed
        {
            fn name(&self) -> String {
                self.state_name()
            }

            #async_modifier fn next(self: Box<Self>) -> #transition_type<#workflow_lifetime, #result_type> {
                #state_trait_ident::next(self) #await_operator
            }
        }
    })
}
