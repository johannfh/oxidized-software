mod graph;

use graph::graph_impl;
use proc_macro::TokenStream;

#[proc_macro]
pub fn graph(input: TokenStream) -> TokenStream {
    graph_impl(input)
}
