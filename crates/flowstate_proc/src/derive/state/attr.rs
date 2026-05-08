use syn::{
    Expr, Ident, Token,
    parse::{Parse, ParseStream},
};

use crate::err::{DuplicateAttributeArgument, UnknownAttributeArgument};

pub struct FlowstateAttrArgs {
    pub name_expr: Option<Expr>,
}

impl Parse for FlowstateAttrArgs {
    fn parse(mut input: ParseStream) -> syn::Result<Self> {
        let mut name_expr = None;

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
                _ => {
                    return Err(UnknownAttributeArgument::at(ident).with(ident_name).into());
                }
            }
        }

        Ok(FlowstateAttrArgs { name_expr })
    }
}

impl FlowstateAttrArgs {
    fn parse_name_expr(input: &mut ParseStream) -> syn::Result<Expr> {
        input.parse::<Token![=]>()?;
        input.parse::<Expr>()
    }
}
