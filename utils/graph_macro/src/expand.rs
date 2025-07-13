use std::collections::HashSet;

use crate::ast::{ConnectionDef, GraphInput};
use proc_macro::TokenStream;
use quote::quote;
use syn::Error as SynError;

pub fn proc_macro(input: &GraphInput) -> TokenStream {
    let mut generated_output = quote! {
        let mut graph = ::utils::graph::Graph::new();
    };

    match input {
        GraphInput::Empty => {
            // Nothing extra to generate
        }
        GraphInput::SimpleNodes(nodes) => {
            let mut node_inserts = quote! {};
            let mut node_names: HashSet<String> = HashSet::new();
            for node_def in nodes {
                let name = &node_def.name;
                let value = &node_def.value;
                let name_str = name.to_string();

                if !node_names.insert(name_str.clone()) {
                    return SynError::new_spanned(
                        name,
                        format!("Duplicate node identifier: `{}`", name_str),
                    )
                    .to_compile_error()
                    .into();
                }

                node_inserts.extend(quote! {
                    graph.insert(::utils::graph::Node::new(#value));
                });
            }
            generated_output.extend(node_inserts);
        }
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
                    return SynError::new_spanned(
                        name,
                        format!("Duplicate node identifier: `{}`)", name_str),
                    )
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
                    ConnectionDef::Directed {
                        from,
                        _arrow_token,
                        to,
                        weight,
                    } => {
                        let weight_val = weight
                            .clone()
                            .map(|(_, l)| l.base10_parse().unwrap_or(1.0))
                            .unwrap_or(1.0);
                        (from, to, quote! { #weight_val }, false)
                    }
                    ConnectionDef::Undirected {
                        from,
                        _double_arrow_token,
                        to,
                        weight,
                    } => {
                        let weight_val = weight
                            .clone()
                            .map(|(_, l)| l.base10_parse().unwrap_or(1.0))
                            .unwrap_or(1.0);
                        (from, to, quote! { #weight_val }, true)
                    }
                };

                // --- Compile-time check for node existence ---
                let from_name_str = from_ident.to_string();
                if !node_names.contains(&from_name_str) {
                    return SynError::new_spanned(
                        from_ident,
                        format!("Node '{}' not defined in `nodes` block.", from_name_str),
                    )
                    .to_compile_error()
                    .into();
                }
                let to_name_str = to_ident.to_string();
                if !node_names.contains(&to_name_str) {
                    return SynError::new_spanned(
                        to_ident,
                        format!("Node '{}' not defined in `nodes` block.", to_name_str),
                    )
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
        }
    }

    quote! {{
        #generated_output
        graph
    }}
    .into()
}
