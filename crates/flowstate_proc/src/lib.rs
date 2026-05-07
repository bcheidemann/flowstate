use syn::{DeriveInput, parse_macro_input};

use crate::derive::derive_workflow_inner;

mod derive;
mod err;

#[proc_macro_derive(Workflow, attributes(flowstate, state))]
pub fn derive_workflow(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    derive_workflow_inner(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
