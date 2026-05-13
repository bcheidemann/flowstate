use syn::{
    Expr, Ident, Token,
    parse::{Parse, ParseStream},
};

use crate::{
    derive::common::FieldAssignment,
    err::{DuplicateAttributeArgument, UnknownAttributeArgument},
};

pub struct FlowstateAttrArgs {
    pub name_expr: Option<Expr>,
    pub ctx_key_value_pairs: Vec<FieldAssignment>,
}

impl Parse for FlowstateAttrArgs {
    fn parse(mut input: ParseStream) -> syn::Result<Self> {
        let mut name_expr = None;
        let mut ctx_key_value_pairs = Vec::new();

        while !input.is_empty() {
            let ident = input.parse::<Ident>()?;
            let ident_name = ident.to_string();

            match ident_name.as_str() {
                "name" => {
                    if name_expr.is_some() {
                        return Err(DuplicateAttributeArgument::at(ident).with("result").into());
                    }
                    name_expr = Some(Self::parse_name_expr(&mut input)?)
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

        Ok(FlowstateAttrArgs {
            name_expr,
            ctx_key_value_pairs,
        })
    }
}

impl FlowstateAttrArgs {
    fn parse_name_expr(input: &mut ParseStream) -> syn::Result<Expr> {
        input.parse::<Token![=]>()?;
        input.parse::<Expr>()
    }
}
