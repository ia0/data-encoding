//! Internal library for data-encoding-macro
//!
//! Do **not** use this library. Use [data-encoding-macro] instead.
//!
//! This library is for internal use by data-encoding-macro because procedural
//! macros require a separate crate.
//!
//! [data-encoding-macro]: https://crates.io/crates/data-encoding-macro

#![feature(proc_macro)]
#![warn(unused_results)]

extern crate proc_macro;
extern crate syn;

extern crate data_encoding;

use proc_macro::{Spacing, TokenNode, TokenStream, TokenTree, TokenTreeIter};
use std::collections::HashMap;

use data_encoding::{BitOrder, Encoding, Specification, Translate, Wrap};

fn parse_op(tokens: &mut TokenTreeIter, op: char, key: &str) {
    match tokens.next() {
        Some(TokenTree { span: _, kind: TokenNode::Op(x, Spacing::Alone) })
            if x == op => (),
        _ => panic!("expected {:?} after {}", op, key),
    }
}

fn parse_map(mut tokens: TokenTreeIter) -> HashMap<String, TokenNode> {
    let mut map = HashMap::new();
    while let Some(key) = tokens.next() {
        let key = match key.kind {
            TokenNode::Term(term) => term.as_str().to_string(),
            _ => panic!("expected key got {}", key),
        };
        parse_op(&mut tokens, ':', &key);
        let value = match tokens.next() {
            None => panic!("expected value for {}", key),
            Some(value) => value.kind,
        };
        parse_op(&mut tokens, ',', &key);
        let _ = map.insert(key, value);
    }
    map
}

fn get_string(map: &mut HashMap<String, TokenNode>, key: &str) -> String {
    let node = match map.remove(key) {
        None => return String::new(),
        Some(node) => node,
    };
    let literal = match node {
        TokenNode::Literal(literal) => literal,
        _ => panic!("expected literal for {}", key),
    };
    match syn::parse::string(&literal.to_string()) {
        syn::parse::IResult::Done(_, result) => result.value,
        _ => panic!("expected string for {}", key),
    }
}

fn get_usize(map: &mut HashMap<String, TokenNode>, key: &str) -> usize {
    let node = match map.remove(key) {
        None => return 0,
        Some(node) => node,
    };
    let literal = match node {
        TokenNode::Literal(literal) => literal,
        _ => panic!("expected literal for {}", key),
    };
    match literal.to_string().parse() {
        Ok(result) => result,
        Err(error) => panic!("expected usize for {}: {}", key, error),
    }
}

fn get_padding(map: &mut HashMap<String, TokenNode>) -> Option<char> {
    let node = match map.remove("padding") {
        None => return None,
        Some(node) => node,
    };
    let literal = match node {
        TokenNode::Term(term) if term.as_str() == "None" => return None,
        TokenNode::Literal(literal) => literal,
        _ => panic!("expected literal for padding"),
    };
    Some(syn::parse::character(&literal.to_string()).expect(
        "expected char for padding",
    ))
}

fn get_bool(map: &mut HashMap<String, TokenNode>, key: &str) -> Option<bool> {
    let node = match map.remove(key) {
        None => return None,
        Some(node) => node,
    };
    let term = match node {
        TokenNode::Term(term) => term,
        _ => panic!("expected literal for padding"),
    };
    Some(syn::parse::boolean(term.as_str()).expect(
        "expected bool for padding",
    ))
}

fn get_bit_order(map: &mut HashMap<String, TokenNode>) -> BitOrder {
    let node = match map.remove("bit_order") {
        None => return BitOrder::MostSignificantFirst,
        Some(node) => node,
    };
    let msb = "MostSignificantFirst";
    let lsb = "LeastSignificantFirst";
    match node {
        TokenNode::Term(term) if term.as_str() == msb => {
            BitOrder::MostSignificantFirst
        }
        TokenNode::Term(term) if term.as_str() == lsb => {
            BitOrder::LeastSignificantFirst
        }
        _ => panic!("expected {} or {} for bit_order", msb, lsb),
    }
}

fn check_present(hash_map: &HashMap<String, TokenNode>, key: &str) {
    if !hash_map.contains_key(key) {
        panic!("{} is required", key);
    }
}

fn get_encoding(mut hash_map: &mut HashMap<String, TokenNode>) -> Encoding {
    check_present(&hash_map, "symbols");
    let spec = Specification {
        symbols: get_string(&mut hash_map, "symbols"),
        bit_order: get_bit_order(&mut hash_map),
        check_trailing_bits: get_bool(&mut hash_map, "check_trailing_bits")
            .unwrap_or(true),
        padding: get_padding(&mut hash_map),
        ignore: get_string(&mut hash_map, "ignore"),
        wrap: Wrap {
            width: get_usize(&mut hash_map, "wrap_width"),
            separator: get_string(&mut hash_map, "wrap_separator"),
        },
        translate: Translate {
            from: get_string(&mut hash_map, "translate_from"),
            to: get_string(&mut hash_map, "translate_to"),
        },
    };
    spec.encoding().unwrap()
}

fn check_empty(hash_map: HashMap<String, TokenNode>) {
    if !hash_map.is_empty() {
        panic!("Unexpected keys {:?}", hash_map.keys());
    }
}

#[proc_macro]
pub fn internal_new_encoding(input: TokenStream) -> TokenStream {
    let mut hash_map = parse_map(input.into_iter());
    let encoding = get_encoding(&mut hash_map);
    check_empty(hash_map);
    format!("{:?}", encoding.internal_implementation()).parse().unwrap()
}

#[proc_macro]
pub fn internal_decode(input: TokenStream) -> TokenStream {
    let mut hash_map = parse_map(input.into_iter());
    let encoding = get_encoding(&mut hash_map);
    check_present(&mut hash_map, "name");
    let name = get_string(&mut hash_map, "name");
    check_present(&mut hash_map, "input");
    let input = get_string(&mut hash_map, "input");
    check_empty(hash_map);
    let output = encoding.decode(input.as_bytes()).unwrap();
    format!("{}: [u8; {}] = {:?};", name, output.len(), output).parse().unwrap()
}
