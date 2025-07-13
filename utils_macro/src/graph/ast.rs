use syn::{
    Expr, Ident, LitFloat, Result, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token,
};

syn::custom_punctuation!(DoubleArrow, <->);

#[derive(Debug)]
pub(super) struct NodeDef {
    pub name: Ident,
    _equal_token: Token![=],
    pub value: Expr,
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
pub(super) enum ConnectionDef {
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
pub(super) enum GraphInput {
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
