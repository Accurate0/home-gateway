mod lua_class;
mod lua_module;
mod workflow_context_variables;

use proc_macro::TokenStream;
use syn::{DeriveInput, ItemImpl, parse_macro_input};

#[proc_macro_derive(WorkflowContextVariables, attributes(variables))]
pub fn derive_workflow_context_variables(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match workflow_context_variables::expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

#[proc_macro_derive(LuaClass, attributes(lua))]
pub fn derive_lua_class(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match lua_class::expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn lua_module(arguments: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemImpl);

    match lua_module::expand(arguments.into(), input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}
