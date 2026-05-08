mod attr;

use quote::quote;
use syn::{DeriveInput, Expr, Ident};

use crate::derive::state::attr::FlowstateAttrArgs;

pub fn derive_state_impl(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let s = validate_input(&input)?;

    impl_state(s)
}

struct ValidatedStateStruct<'s> {
    ident: &'s Ident,
    name_expr: Option<Expr>,
}

fn validate_input(input: &DeriveInput) -> syn::Result<ValidatedStateStruct<'_>> {
    let attr = validate_flowstate_attr(input)?;

    Ok(ValidatedStateStruct {
        ident: &input.ident,
        name_expr: attr.and_then(|attr| attr.name_expr),
    })
}

fn validate_flowstate_attr(input: &DeriveInput) -> syn::Result<Option<FlowstateAttrArgs>> {
    let Some(attr) = input.attrs.iter().find(|a| a.path().is_ident("flowstate")) else {
        return Ok(None);
    };

    attr.parse_args().map(Some)
}

fn impl_state(s: ValidatedStateStruct<'_>) -> syn::Result<proc_macro2::TokenStream> {
    let ValidatedStateStruct { ident, name_expr } = s;

    let name_expr = match name_expr {
        Some(_) => quote! { #name_expr.into() },
        None => quote! { ::std::any::type_name::<#ident>().to_string() },
    };

    Ok(quote! {
        impl ::flowstate::State for #ident {
            fn name(&self) -> String {
                #name_expr
            }
        }
    })
}
