use syn::{DeriveInput, parse_macro_input};

use crate::derive::{state::derive_state_impl, workflow::derive_workflow_impl};

mod derive;
mod err;

#[proc_macro_derive(State, attributes(flowstate))]
pub fn derive_state(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    derive_state_impl(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_derive(Workflow, attributes(flowstate, state))]
pub fn derive_workflow(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    derive_workflow_impl(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
