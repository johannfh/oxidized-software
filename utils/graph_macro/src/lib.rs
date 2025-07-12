use proc_macro::TokenStream;
use syn::{
    parse::{Parse, ParseStream}, punctuated::Punctuated, token, Expr, Ident, LitFloat, Result, Token
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
        _connections_keyword: Ident,
        _connections_colon: Token![:],
        _connections_brace_open: token::Brace,
        // ident -> ident, ident <-> ident, ...
        connections: Punctuated<ConnectionDef, Token![;]>,
        _connections_brace_close: token::Brace,
    },
}

impl Parse for GraphInput {
    fn parse(input: ParseStream) -> Result<Self> {
        todo!()
    }
}


#[proc_macro]
pub fn graph(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as GraphInput);

    println!("graph input: {:?}", input);

    todo!()
}
