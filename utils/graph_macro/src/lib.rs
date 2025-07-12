use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream}, punctuated::Punctuated, token, Error, Expr, Ident, LitFloat, Result, Token
};

mod custom_syntax {
    syn::custom_punctuation!(DoubleArrow, <->);
}

use custom_syntax::*;

#[derive(Debug)]
struct NodeDef {
    name: Ident,
    _equal_token: Token![=],
    value: Expr,
}

impl Parse for NodeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        let colon_token = input.parse()?;
        let value = input.parse()?;

        Ok(Self {
            name,
            _equal_token: colon_token,
            value,
        })
    }
}

#[derive(Debug)]
enum ConnectionDef {
    Directed {
        from: Ident,
        _arrow_token: Token![->],
        to: Ident,
        weight: Option<(Token![:], LitFloat)>,
    },
    Undirected {
        from: Ident,
        _double_arrow_token: DoubleArrow,
        to: Ident,
        weight: Option<(Token![:], LitFloat)>,
    },
}

impl Parse for ConnectionDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let from: Ident = input.parse()?;

        if input.peek(Token![->]) {
            let arrow_token = input.parse()?;
            let to = input.parse()?;

            let weight = if input.peek(Token![:]) {
                Some((input.parse()?, input.parse()?))
            } else {
                None
            };

            Ok(Self::Directed {
                from,
                _arrow_token: arrow_token,
                to,
                weight,
            })
        } else if input.peek(DoubleArrow) {
            let double_arrow_token = input.parse()?;
            let to = input.parse()?;

            let weight = if input.peek(Token![:]) {
                Some((input.parse()?, input.parse()?))
            } else {
                None
            };

            Ok(Self::Undirected {
                from,
                _double_arrow_token: double_arrow_token,
                to,
                weight,
            })
        } else {
            Err(input.error("expected `->` or `<->` connection"))
        }
    }
}

#[derive(Debug)]
enum GraphInput {
    Empty,
    SimpleNodes(Punctuated<NodeDef, Token![,]>),
    Structured {
        // nodes: { ... },
        _nodes_keyword: Ident,
        _nodes_colon: Token![:],
        _nodes_brace_open: token::Brace,
        // ident = expr, ident = expr, ...
        nodes: Punctuated<NodeDef, Token![;]>,
        _nodes_brace_close: token::Brace,

        // connections: { ... },
        _connections_keyword: Option<Ident>,
        _connections_colon: Option<Token![:]>,
        _connections_brace_open: Option<token::Brace>,
        /// ident -> ident, ident <-> ident, ...
        connections: Punctuated<ConnectionDef, Token![;]>,
        _connections_brace_close: Option<token::Brace>,
    },
}

impl Parse for GraphInput {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Ok(Self::Empty);
        }

        let fork = input.fork();
        if fork.peek(Ident) && fork.peek2(Token![:]) && fork.peek3(token::Brace) {
            let first_ident: Ident = fork.parse()?;
            if first_ident.to_string() == "nodes" {
                let nodes_keyword: Ident = input.parse()?;
                let nodes_colon: Token![:] = input.parse()?;
                let nodes_conent;
                let nodes_brace_open = syn::braced!(nodes_conent in input);
                // parse complete content of braces
                // nodes: { <content> }
                let nodes = nodes_conent.parse_terminated(NodeDef::parse, Token![;])?;
                let nodes_brace_close = nodes_brace_open;

                let mut connections_keyword = None;
                let mut connections_colon = None;
                let mut connections_brace_open = None;
                let mut connections = Punctuated::new();
                let mut connections_brace_close = None;

                if input.peek(Ident) {
                    let fork = input.fork();
                    let conn_ident: Ident = fork.parse()?;
                    if conn_ident.to_string() == "connections" {
                        connections_keyword = Some(input.parse()?);
                        connections_colon = Some(input.parse()?);
                        let connections_content;
                        connections_brace_open = Some(syn::braced!(connections_content in input));
                        // parse complete content of braces
                        // connections: { <content> }
                        connections = connections_content
                            .parse_terminated(ConnectionDef::parse, Token![;])?;
                        connections_brace_close = connections_brace_open;
                    }
                }

                return Ok(GraphInput::Structured {
                    _nodes_keyword: nodes_keyword,
                    _nodes_colon: nodes_colon,
                    _nodes_brace_open: nodes_brace_open,
                    nodes,
                    _nodes_brace_close: nodes_brace_close,
                    _connections_keyword: connections_keyword,
                    _connections_colon: connections_colon,
                    _connections_brace_open: connections_brace_open,
                    connections,
                    _connections_brace_close: connections_brace_close,
                });
            }
        }

        let simple_nodes = input.parse_terminated(NodeDef::parse, Token![,])?;
        Ok(GraphInput::SimpleNodes(simple_nodes))
    }
}

#[proc_macro]
pub fn graph(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as GraphInput);

    let mut generated_output = quote! {

        let mut graph = ::utils::graph::Graph::new();
    };

    match input {
        GraphInput::Empty => {
            // Nothing extra to generate
        },
        GraphInput::SimpleNodes(nodes) => {
            let mut node_inserts = quote! {};
            let mut node_names: HashSet<String> = HashSet::new();
            for node_def in nodes {
                let name = &node_def.name;
                let value = &node_def.value;
                let name_str = name.to_string();

                if !node_names.insert(name_str.clone()) {
                    return Error::new_spanned(name, format!("Duplicate node identifier: `{}`", name_str))
                        .to_compile_error()
                        .into();
                }

                node_inserts.extend(quote! {
                    graph.insert(::utils::graph::Node::new(#value));
                });
            }
            generated_output.extend(node_inserts);
        },
        GraphInput::Structured {
            nodes, connections, ..
        } => {
            let id_map_init = quote! {
                let mut id_map: ::std::collections::HashMap<&'static str, ::utils::graph::NodeID> = ::std::collections::HashMap::new();
            };
            let mut node_inserts = quote! {};
            let mut node_names: HashSet<String> = HashSet::new();
            for node_def in nodes {
                let name = &node_def.name;
                let value = &node_def.value;
                let name_str = name.to_string();

                if !node_names.insert(name_str.clone()) {
                    return Error::new_spanned(name, format!("Duplicate node identifier: `{}`)", name_str))
                        .to_compile_error()
                        .into();
                }

                node_inserts.extend(quote! {
                    let node_id = graph.insert(::utils::graph::Node::new(#value));
                    id_map.insert(stringify!(#name), node_id);
                });
            }

            generated_output.extend(id_map_init);
            generated_output.extend(node_inserts);

            let mut connection_inserts = quote! {};
            for conn_def in connections {
                let (from_ident, to_ident, weight_expr, is_undirected) = match conn_def {
                    ConnectionDef::Directed { from, _arrow_token, to, weight } => {
                        let weight_val = weight.map(|(_, l)| l.base10_parse().unwrap_or(1.0)).unwrap_or(1.0);
                        (from, to, quote! { #weight_val }, false)
                    },
                    ConnectionDef::Undirected { from, _double_arrow_token, to, weight } => {
                        let weight_val = weight.map(|(_, l)| l.base10_parse().unwrap_or(1.0)).unwrap_or(1.0);
                        (from, to, quote! { #weight_val }, true)
                    },
                };

                // --- Compile-time check for node existence ---
                let from_name_str = from_ident.to_string();
                if !node_names.contains(&from_name_str) {
                    return Error::new_spanned(from_ident, format!("Node '{}' not defined in `nodes` block.", from_name_str))
                        .to_compile_error()
                        .into();
                }
                let to_name_str = to_ident.to_string();
                if !node_names.contains(&to_name_str) {
                    return Error::new_spanned(to_ident, format!("Node '{}' not defined in `nodes` block.", to_name_str))
                        .to_compile_error()
                        .into();
                }
                // --- End compile-time check ---
                
                connection_inserts.extend(quote! {
                    let from_id = *id_map.get(stringify!(#from_ident)).expect("Internal macro error: Node not found after compile-time check.");
                    let to_id = *id_map.get(stringify!(#to_ident)).expect("Internal macro error: Node not found after compile-time check.");
                    graph.connect(from_id, to_id, #weight_expr);
                });
                
                if is_undirected {
                    connection_inserts.extend(quote! {
                        let from_id = *id_map.get(stringify!(#from_ident)).expect("Internal macro error: Node not found after compile-time check.");
                        let to_id = *id_map.get(stringify!(#to_ident)).expect("Internal macro error: Node not found after compile-time check.");
                        graph.connect(to_id, from_id, #weight_expr);
                    });
                }
            }

            generated_output.extend(connection_inserts);
        },
    }

    quote! {{
        #generated_output
        graph
    }}
    .into()
}
