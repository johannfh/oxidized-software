mod ast;
mod expand;

use ast::GraphInput;
use proc_macro::TokenStream;
use syn::parse_macro_input;

pub(super) fn graph_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GraphInput);
    expand::expand(input)
}
