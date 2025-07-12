#[macro_export]
macro_rules! __graph_connection_parser {
    // Base case: No more connections to parse
    ($graph:ident, $id_map:ident,) => {};

    // Case: Directed connection with explicit weight (e.g. a -> b: 5.0;)
    ($graph:ident, $id_map:ident, $from:ident -> $to:ident : $weight:expr; $($rest:tt)*) => {
        let from_id = *$id_map.get(stringify!($from))
            .expect(&format!("Node '{}' not found for connection", stringify!($from)));
        let to_id = *$id_map.get(stringify!($to))
            .expect(&format!("Node '{}' not found for connection", stringify!($to)));
        $graph.connect(from_id, to_id, $weight as f64);
        // Recursively call with the remaining tokens
        __graph_connection_parser! { $graph, $id_map, $($rest)* }
    };

    // Case: Undirected connection with explicit weight (e.g. a <-> b: 5.0;)
    ($graph:ident, $id_map:ident, $from:ident <-> $to:ident : $weight:expr; $($rest:tt)*) => {
        let from_id = *$id_map.get(stringify!($from))
            .expect(&format!("Node '{}' not found for connection", stringify!($from)));
        let to_id = *$id_map.get(stringify!($to))
            .expect(&format!("Node '{}' not found for connection", stringify!($to)));
        $graph.connect_bidirectionally(from_id, to_id, $weight as f64);
        // Recursively call with the remaining tokens
        __graph_connection_parser! { $graph, $id_map, $($rest)* }
    };
}

#[macro_export]
macro_rules! graph {
    () => {
        crate::graph::Graph::new()
    };

    ($($name:ident = $value:expr),+ $(,)?) => {
        {
            use crate::graph::{Graph, Node};

            let mut graph = Graph::new();

            $(
                graph.insert(Node::new($value));
            )+

            graph
        }
    };

    (
        nodes: { $($node_name:ident = $node_value:expr,)* } $(,)?
        connections: { $($connection:tt)* } $(,)?
    ) => {
        {
            use ::std::collections::HashMap;
            use ::utils::graph::{Graph, NodeID, Node};
            use ::utils::__graph_connection_parser;

            let mut graph = Graph::new();

            let mut id_map: HashMap<&'static str, NodeID> = HashMap::new();

            // process nodes block
            $(
                let node_id = graph.insert(Node::new($node_value));
                id_map.insert(stringify!($node_name), node_id);
            )*


            // process connections block
            __graph_connection_parser! {
                graph,
                id_map,
                $($connection)*
            }

            graph
        }
    };
}
