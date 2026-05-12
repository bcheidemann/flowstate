mod attr;

use quote::quote;
use syn::{DeriveInput, Generics, Ident};

use crate::derive::state::attr::FlowstateAttrArgs;

pub fn derive_state_impl(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let s = validate_input(&input)?;

    impl_state(s)
}

struct ValidatedStateStruct<'s> {
    ident: &'s Ident,
    args: Option<FlowstateAttrArgs>,
    generics: &'s Generics,
}

fn validate_input(input: &DeriveInput) -> syn::Result<ValidatedStateStruct<'_>> {
    let args = validate_flowstate_attr(input)?;
    let generics = &input.generics;

    Ok(ValidatedStateStruct {
        ident: &input.ident,
        args,
        generics,
    })
}

fn validate_flowstate_attr(input: &DeriveInput) -> syn::Result<Option<FlowstateAttrArgs>> {
    let Some(attr) = input.attrs.iter().find(|a| a.path().is_ident("flowstate")) else {
        return Ok(None);
    };

    attr.parse_args().map(Some)
}

fn impl_state(s: ValidatedStateStruct<'_>) -> syn::Result<proc_macro2::TokenStream> {
    let ident = s.ident;
    let generics = s.generics;
    let const_params = s.generics.const_params().collect::<Vec<_>>();
    let lifetime_params = s.generics.lifetimes().collect::<Vec<_>>();
    let type_params = s.generics.type_params().collect::<Vec<_>>();
    let generic_args_bracketed = {
        let mut params = Vec::new();

        params.extend(lifetime_params.iter().map(|param| {
            let param = &param.lifetime;
            quote! { #param }
        }));
        params.extend(const_params.iter().map(|param| {
            let ident = &param.ident;
            quote! { #ident }
        }));
        params.extend(type_params.iter().map(|param| {
            let ident = &param.ident;
            quote! { #ident }
        }));

        if params.is_empty() {
            None
        } else {
            Some(quote! {
                <#(#params,)*>
            })
        }
    };
    let where_clause = &s.generics.where_clause;
    let name_expr = match s.args {
        Some(FlowstateAttrArgs {
            name_expr: Some(name_expr),
        }) => quote! { #name_expr.into() },
        _ => quote! {
            ::std::any::type_name::<#ident #generic_args_bracketed>().to_string()
        },
    };

    Ok(quote! {
        impl #generics ::flowstate::State for #ident #generic_args_bracketed #where_clause {
            fn name(&self) -> String {
                #name_expr
            }
        }
    })
}
