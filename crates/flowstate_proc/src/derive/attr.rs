use syn::{
    Ident, Path, Token,
    parse::{Parse, ParseStream},
};

use crate::err::{
    DuplicateResultArguments, MissingResultArgument, UnexpectedArgumentsForStateAttribute,
    UnknownArgument,
};

pub struct FlowstateAttr {
    pub result_type: Path,
}

impl Parse for FlowstateAttr {
    fn parse(mut input: ParseStream) -> syn::Result<Self> {
        let span = input.span();

        let mut result_type = None;

        while !input.is_empty() {
            let ident = input.parse::<Ident>()?;
            let ident_name = ident.to_string();

            match ident_name.as_str() {
                "result" => {
                    if result_type.is_some() {
                        return Err(DuplicateResultArguments::at(ident).into());
                    }
                    result_type = Some(Self::parse_result_type(&mut input)?);
                }
                _ => {
                    return Err(UnknownArgument::at(ident).with(ident_name).into());
                }
            }
        }

        let Some(result_type) = result_type else {
            return Err(MissingResultArgument::at(span).into_syn_error());
        };

        Ok(Self { result_type })
    }
}

impl FlowstateAttr {
    fn parse_result_type(input: &mut ParseStream) -> syn::Result<Path> {
        input.parse::<Token![=]>()?;
        input.parse::<Path>()
    }
}

pub struct StateAttr;

impl Parse for StateAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if !input.is_empty() {
            let unexpected: proc_macro2::TokenStream = input.parse()?;
            return Err(UnexpectedArgumentsForStateAttribute::at(unexpected).into());
        }

        Ok(Self)
    }
}
