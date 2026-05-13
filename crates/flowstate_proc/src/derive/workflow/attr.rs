use syn::{
    Expr, Ident, Path, Token,
    parse::{Parse, ParseStream},
};

use crate::{
    derive::common::FieldAssignment,
    err::{
        DeprecatedIsAsync, DuplicateAttributeArgument, MissingAttributeArgument,
        UnexpectedArgumentsForStateAttribute, UnknownAttributeArgument,
    },
};

pub struct FlowstateAttrArgs {
    pub result_type: Path,
    pub state_trait_ident: Option<Ident>,
    pub name_expr: Option<Expr>,
    pub ctx_key_value_pairs: Vec<FieldAssignment>,
}

impl Parse for FlowstateAttrArgs {
    fn parse(mut input: ParseStream) -> syn::Result<Self> {
        let span = input.span();

        let mut result_type = None;
        let mut state_trait_ident = None;
        let mut name_expr = None;
        let mut ctx_key_value_pairs = Vec::new();

        while !input.is_empty() {
            let ident = input.parse::<Ident>()?;
            let ident_name = ident.to_string();

            match ident_name.as_str() {
                "is_async" => {
                    return Err(DeprecatedIsAsync::at(ident).into());
                }
                "result" => {
                    if result_type.is_some() {
                        return Err(DuplicateAttributeArgument::at(ident).with("result").into());
                    }
                    result_type = Some(Self::parse_result_type(&mut input)?);
                }
                "state_trait" => {
                    if state_trait_ident.is_some() {
                        return Err(DuplicateAttributeArgument::at(ident)
                            .with("state_trait")
                            .into());
                    }
                    state_trait_ident = Some(Self::parse_state_trait_ident(&mut input)?);
                }
                "name" => {
                    if name_expr.is_some() {
                        return Err(DuplicateAttributeArgument::at(ident).with("name").into());
                    }
                    name_expr = Some(Self::parse_name_expr(&mut input)?);
                }
                "ctx" => {
                    input.parse::<Token![.]>()?;
                    ctx_key_value_pairs.push(FieldAssignment::parse(&mut input)?);
                }
                _ => {
                    return Err(UnknownAttributeArgument::at(ident).with(ident_name).into());
                }
            }

            if input.is_empty() {
                break;
            }

            input.parse::<Token![,]>()?;
        }

        let Some(result_type) = result_type else {
            return Err(MissingAttributeArgument::at(span)
                .with("result")
                .into_syn_error());
        };

        Ok(Self {
            result_type,
            state_trait_ident,
            name_expr,
            ctx_key_value_pairs,
        })
    }
}

impl FlowstateAttrArgs {
    fn parse_result_type(input: &mut ParseStream) -> syn::Result<Path> {
        input.parse::<Token![=]>()?;
        input.parse::<Path>()
    }

    fn parse_state_trait_ident(input: &mut ParseStream) -> syn::Result<Ident> {
        input.parse::<Token![=]>()?;
        input.parse::<Ident>()
    }

    fn parse_name_expr(input: &mut ParseStream) -> syn::Result<Expr> {
        input.parse::<Token![=]>()?;
        input.parse::<Expr>()
    }
}

pub struct StateAttrArgs;

impl Parse for StateAttrArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if !input.is_empty() {
            let unexpected: proc_macro2::TokenStream = input.parse()?;
            return Err(UnexpectedArgumentsForStateAttribute::at(unexpected).into());
        }

        Ok(Self)
    }
}
