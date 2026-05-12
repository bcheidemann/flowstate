use syn::{
    Expr, Ident, LitBool, Path, Token,
    parse::{Parse, ParseStream},
};

use crate::err::{
    DuplicateAttributeArgument, MissingAttributeArgument, UnexpectedArgumentsForStateAttribute,
    UnknownAttributeArgument,
};

pub struct FlowstateAttrArgs {
    pub is_async: bool,
    pub result_type: Path,
    pub state_trait_ident: Option<Ident>,
    pub name_expr: Option<Expr>,
}

impl Parse for FlowstateAttrArgs {
    fn parse(mut input: ParseStream) -> syn::Result<Self> {
        let span = input.span();

        let mut is_async = None;
        let mut result_type = None;
        let mut state_trait_ident = None;
        let mut name_expr = None;

        while !input.is_empty() {
            let ident = input.parse::<Ident>()?;
            let ident_name = ident.to_string();

            match ident_name.as_str() {
                "is_async" => {
                    if is_async.is_some() {
                        return Err(DuplicateAttributeArgument::at(ident).with("async").into());
                    }
                    is_async = Some(Self::parse_is_async(&mut input)?);
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
            is_async: is_async.unwrap_or(false),
            result_type,
            state_trait_ident,
            name_expr,
        })
    }
}

impl FlowstateAttrArgs {
    fn parse_is_async(input: &mut ParseStream) -> syn::Result<bool> {
        if !input.peek(Token![=]) {
            return Ok(true);
        }
        input.parse::<Token![=]>()?;
        let value = input.parse::<LitBool>()?;
        Ok(value.value)
    }

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
