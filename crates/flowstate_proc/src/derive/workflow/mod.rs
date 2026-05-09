mod attr;

use quote::quote;
use syn::{DeriveInput, Fields, FieldsNamed, Ident, Path, Type, TypeParam};

use crate::{
    derive::workflow::attr::{FlowstateAttrArgs, StateAttrArgs},
    err::{
        DuplicateStateAttribute, InvalidStateFieldType, MissingFlowstateAttributeOnWorkflow,
        MissingGenericTypeParameterForState, MissingStateAttribute, UnexpectedUnnamedField,
        UnsupportedAdditionalGenericTypeParameters,
        UnsupportedBoundsOnGenericTypeParameterForState, UnsupportedEnumOrUnion,
        UnsupportedGenericConstParameter, UnsupportedGenericLifetimeParameter,
        UnsupportedGenericWhereClause, UnsupportedTupleStruct, UnsupportedUnitStruct,
    },
};

pub fn derive_workflow_impl(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let s = validate_input(&input)?;

    impl_workflow(s)
}

struct ValidatedWorkflowStruct<'s> {
    ident: &'s Ident,
    result_type: Path,
    fields: ValidatedStructFields<'s>,
}

fn validate_input(input: &DeriveInput) -> syn::Result<ValidatedWorkflowStruct<'_>> {
    let attr = validate_flowstate_attr(input)?;
    let state_param = validate_state_type_param(input)?;
    let fields = validate_struct(input, state_param)?;

    Ok(ValidatedWorkflowStruct {
        ident: &input.ident,
        result_type: attr.result_type,
        fields,
    })
}

fn validate_flowstate_attr(input: &DeriveInput) -> syn::Result<FlowstateAttrArgs> {
    let Some(attr) = input.attrs.iter().find(|a| a.path().is_ident("flowstate")) else {
        return Err(MissingFlowstateAttributeOnWorkflow::at(&input.ident).into());
    };

    attr.parse_args()
}

fn validate_state_type_param(input: &DeriveInput) -> syn::Result<&TypeParam> {
    if let Some(lifetime) = input.generics.lifetimes().next() {
        return Err(UnsupportedGenericLifetimeParameter::at(lifetime).into());
    }

    if let Some(const_param) = input.generics.const_params().next() {
        return Err(UnsupportedGenericConstParameter::at(const_param).into());
    }

    let mut type_params = input.generics.type_params();

    let Some(state_param) = type_params.next() else {
        return Err(MissingGenericTypeParameterForState::at(&input.ident).into());
    };

    if let Some(extra_type_param) = type_params.next() {
        return Err(UnsupportedAdditionalGenericTypeParameters::at(&extra_type_param).into());
    }

    if let Some(bound) = state_param.bounds.first() {
        return Err(UnsupportedBoundsOnGenericTypeParameterForState::at(bound).into());
    }

    if let Some(where_clause) = &input.generics.where_clause {
        return Err(UnsupportedGenericWhereClause::at(where_clause).into());
    }

    Ok(state_param)
}

fn validate_struct<'input>(
    input: &'input DeriveInput,
    state_param: &TypeParam,
) -> syn::Result<ValidatedStructFields<'input>> {
    let data = match &input.data {
        syn::Data::Struct(data) => data,
        syn::Data::Enum(_) | syn::Data::Union(_) => {
            return Err(UnsupportedEnumOrUnion::at(&input.ident).into());
        }
    };

    match &data.fields {
        Fields::Named(fields) => validate_struct_fields(fields, state_param),
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
    state_param: &TypeParam,
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
                    .with(ident.to_string(), state_param.ident.to_string())
                    .into());
            };

            let Some(state_ident) = type_path.path.get_ident() else {
                return Err(InvalidStateFieldType::at(type_path)
                    .with(ident.to_string(), state_param.ident.to_string())
                    .into());
            };

            if *state_ident != state_param.ident {
                return Err(InvalidStateFieldType::at(type_path)
                    .with(ident.to_string(), state_param.ident.to_string())
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

fn impl_workflow(s: ValidatedWorkflowStruct) -> syn::Result<proc_macro2::TokenStream> {
    let ValidatedWorkflowStruct {
        ident,
        result_type,
        fields,
    } = s;
    let state_field_ident = fields.state.ident;
    let rest_field_init_params = fields.rest.iter().map(|Field { ident, ty }| {
        quote! {
            #ident: #ty
        }
    });
    let rest_field_idents = fields.rest.iter().map(|Field { ident, .. }| {
        quote! {
            #ident
        }
    });
    let rest_field_transition_assignments: Vec<_> = fields
        .rest
        .iter()
        .map(|Field { ident, .. }| {
            quote! {
                #ident: self.#ident
            }
        })
        .collect();
    let workflow_state_trait_ident = Ident::new(&format!("{}State", ident), ident.span());

    Ok(quote! {
        impl<State> #ident<State> {
            fn new(#state_field_ident: State, #(#rest_field_init_params,)*) -> Self {
                Self {
                    #state_field_ident,
                    #(#rest_field_idents,)*
                }
            }
        }

        impl<State: flowstate::State> ::flowstate::Workflow for #ident<State>
        {
            fn state(&self) -> &dyn ::flowstate::State {
                &self.#state_field_ident
            }
        }

        impl<State> #ident<State>
        {
            fn transition<NextState>(
                self,
                next_state: NextState,
            ) -> ::flowstate::Transition<'static, #result_type>
            where
                #ident<NextState>: ::flowstate::WorkflowState<'static, #result_type> + 'static,
            {
                ::std::ops::ControlFlow::Continue(Box::new(#ident {
                    #state_field_ident: next_state,
                    #(#rest_field_transition_assignments,)*
                }))
            }

            fn transition_with<NextState, Fn>(
                self,
                map_fn: Fn,
            ) -> ::flowstate::Transition<'static, #result_type>
            where
                #ident<NextState>: ::flowstate::WorkflowState<'static, #result_type> + 'static,
                Fn: FnOnce(State) -> NextState,
            {
                Transition::Continue(Box::new(#ident {
                    #state_field_ident: map_fn(self.#state_field_ident),
                    #(#rest_field_transition_assignments,)*
                }))
            }
        }

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
        /// Return [`self.finish(result)`](flowstate::Workflow::finish) or
        /// [`self.finish_with(|workflow| result)`](flowstate::Workflow::finish_with)
        /// to terminate the workflow with a result.
        ///
        trait #workflow_state_trait_ident: ::flowstate::Workflow {
            fn state_name(&self) -> String {
                self.state().name()
            }

            fn next(self: Box<Self>) -> ::flowstate::Transition<'static, #result_type>;
        }

        impl<State> ::flowstate::WorkflowState<'static, #result_type> for #ident<State>
        where
            State: ::flowstate::State,
            #ident<State>: #workflow_state_trait_ident
        {
            fn name(&self) -> String {
                self.state_name()
            }

            fn next(self: Box<Self>) -> ::flowstate::Transition<'static, #result_type> {
                #workflow_state_trait_ident::next(self)
            }
        }
    })
}
