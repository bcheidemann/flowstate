use syn::{
    Ident, Path, Token,
    parse::{Parse, ParseStream},
};

use crate::err::{
    DuplicateAttributeArgument, MissingAttributeArgument, UnexpectedArgumentsForStateAttribute,
    UnknownAttributeArgument,
};

pub struct FlowstateAttrArgs {
    pub result_type: Path,
}

impl Parse for FlowstateAttrArgs {
    fn parse(mut input: ParseStream) -> syn::Result<Self> {
        let span = input.span();

        let mut result_type = None;

        while !input.is_empty() {
            let ident = input.parse::<Ident>()?;
            let ident_name = ident.to_string();

            match ident_name.as_str() {
                "result" => {
                    if result_type.is_some() {
                        return Err(DuplicateAttributeArgument::at(ident).with("result").into());
                    }
                    result_type = Some(Self::parse_result_type(&mut input)?);
                }
                _ => {
                    return Err(UnknownAttributeArgument::at(ident).with(ident_name).into());
                }
            }
        }

        let Some(result_type) = result_type else {
            return Err(MissingAttributeArgument::at(span)
                .with("result")
                .into_syn_error());
        };

        Ok(Self { result_type })
    }
}

impl FlowstateAttrArgs {
    fn parse_result_type(input: &mut ParseStream) -> syn::Result<Path> {
        input.parse::<Token![=]>()?;
        input.parse::<Path>()
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
