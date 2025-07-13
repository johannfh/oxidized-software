mod ast;
mod expand;

use ast::GraphInput;
use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro]
pub fn graph(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GraphInput);
    expand::proc_macro(&input)
}
