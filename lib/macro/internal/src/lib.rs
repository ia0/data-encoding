//! Internal library for data-encoding-macro
//!
//! Do **not** use this library. Use [data-encoding-macro] instead.
//!
//! This library is for internal use by data-encoding-macro because procedural
//! macros require a separate crate.
//!
//! [data-encoding-macro]: https://crates.io/crates/data-encoding-macro

#![warn(unused_results)]

extern crate proc_macro;
#[cfg(feature = "stable")]
#[macro_use]
extern crate proc_macro_hack;
extern crate syn;

extern crate data_encoding;

#[cfg(not(feature = "stable"))]
use proc_macro::token_stream::IntoIter;
#[cfg(not(feature = "stable"))]
use proc_macro::{TokenStream, TokenTree};
use std::collections::HashMap;
#[cfg(feature = "stable")]
use syn::parse::IResult;
#[cfg(feature = "stable")]
use syn::{Lit, Token, TokenTree};

use data_encoding::{BitOrder, Encoding, Specification, Translate, Wrap};

#[cfg(not(feature = "stable"))]
fn parse_op(tokens: &mut IntoIter, op: char, key: &str) {
    match tokens.next() {
        Some(TokenTree::Punct(ref x)) if x.as_char() == op => (),
        _ => panic!("expected {:?} after {}", op, key),
    }
}
#[cfg(feature = "stable")]
fn parse_op<'a>(input: &'a str, op: Token, key: &str) -> &'a str {
    match syn::parse::tt(input) {
        IResult::Done(rest, TokenTree::Token(ref x)) if x == &op => rest,
        _ => panic!("expected {:?} after {}", op, key),
    }
}

#[cfg(not(feature = "stable"))]
fn parse_map(mut tokens: IntoIter) -> HashMap<String, TokenTree> {
    let mut map = HashMap::new();
    while let Some(key) = tokens.next() {
        let key = match key {
            TokenTree::Ident(ident) => format!("{}", ident),
            _ => panic!("expected key got {}", key),
        };
        parse_op(&mut tokens, ':', &key);
        let value = match tokens.next() {
            None => panic!("expected value for {}", key),
            Some(value) => value,
        };
        parse_op(&mut tokens, ',', &key);
        let _ = map.insert(key, value);
    }
    map
}
#[cfg(feature = "stable")]
fn parse_map(mut input: &str) -> HashMap<String, Token> {
    let mut map = HashMap::new();
    loop {
        let key = match syn::parse::tt(input) {
            IResult::Done(rest, key) => {
                input = rest;
                key
            }
            IResult::Error => break,
        };
        let key = match key {
            TokenTree::Token(Token::Ident(key)) => key,
            _ => panic!("expected key got {:?}", key),
        };
        input = parse_op(input, Token::Colon, key.as_ref());
        let value = match syn::parse::tt(input) {
            IResult::Done(rest, TokenTree::Token(value)) => {
                input = rest;
                value
            }
            _ => panic!("expected value for {}", key),
        };
        input = parse_op(input, Token::Comma, key.as_ref());
        let _ = map.insert(key.as_ref().to_string(), value);
    }
    map
}

#[cfg(not(feature = "stable"))]
fn get_string(map: &mut HashMap<String, TokenTree>, key: &str) -> String {
    let node = match map.remove(key) {
        None => return String::new(),
        Some(node) => node,
    };
    let literal = match node {
        TokenTree::Literal(literal) => literal,
        _ => panic!("expected literal for {}", key),
    };
    match syn::parse::string(&literal.to_string()) {
        syn::parse::IResult::Done(_, result) => result.value,
        _ => panic!("expected string for {}", key),
    }
}
#[cfg(feature = "stable")]
fn get_string(map: &mut HashMap<String, Token>, key: &str) -> String {
    match map.remove(key) {
        None => return String::new(),
        Some(Token::Literal(Lit::Str(value, _))) => value,
        Some(_) => panic!("expected string literal for {}", key),
    }
}

#[cfg(not(feature = "stable"))]
fn get_usize(map: &mut HashMap<String, TokenTree>, key: &str) -> usize {
    let node = match map.remove(key) {
        None => return 0,
        Some(node) => node,
    };
    let literal = match node {
        TokenTree::Literal(literal) => literal,
        _ => panic!("expected literal for {}", key),
    };
    match literal.to_string().parse() {
        Ok(result) => result,
        Err(error) => panic!("expected usize for {}: {}", key, error),
    }
}
#[cfg(feature = "stable")]
fn get_usize(map: &mut HashMap<String, Token>, key: &str) -> usize {
    match map.remove(key) {
        None => return 0,
        Some(Token::Literal(Lit::Int(value, _))) => value as usize,
        Some(_) => panic!("expected usize for {}", key),
    }
}

#[cfg(not(feature = "stable"))]
fn get_padding(map: &mut HashMap<String, TokenTree>) -> Option<char> {
    let node = match map.remove("padding") {
        None => return None,
        Some(node) => node,
    };
    let literal = match node {
        TokenTree::Ident(ref ident) if format!("{}", ident) == "None" => return None,
        TokenTree::Literal(literal) => literal,
        _ => panic!("expected literal for padding"),
    };
    Some(syn::parse::character(&literal.to_string()).expect("expected char for padding"))
}
#[cfg(feature = "stable")]
fn get_padding(map: &mut HashMap<String, Token>) -> Option<char> {
    match map.remove("padding") {
        None => return None,
        Some(Token::Ident(ref ident)) if ident.as_ref() == "None" => None,
        Some(Token::Literal(Lit::Char(value))) => Some(value),
        Some(_) => panic!("expected char for padding"),
    }
}

#[cfg(not(feature = "stable"))]
fn get_bool(map: &mut HashMap<String, TokenTree>, key: &str) -> Option<bool> {
    let node = match map.remove(key) {
        None => return None,
        Some(node) => node,
    };
    let ident = match node {
        TokenTree::Ident(ident) => format!("{}", ident),
        _ => panic!("expected literal for padding"),
    };
    Some(syn::parse::boolean(&ident).expect("expected bool for padding"))
}
#[cfg(feature = "stable")]
fn get_bool(map: &mut HashMap<String, Token>, key: &str) -> Option<bool> {
    match map.remove(key) {
        None => return None,
        Some(Token::Literal(Lit::Bool(value))) => Some(value),
        Some(_) => panic!("expected bool for {}", key),
    }
}

#[cfg(not(feature = "stable"))]
fn get_bit_order(map: &mut HashMap<String, TokenTree>) -> BitOrder {
    let node = match map.remove("bit_order") {
        None => return BitOrder::MostSignificantFirst,
        Some(node) => node,
    };
    let msb = "MostSignificantFirst";
    let lsb = "LeastSignificantFirst";
    match node {
        TokenTree::Ident(ref ident) if format!("{}", ident) == msb => {
            BitOrder::MostSignificantFirst
        }
        TokenTree::Ident(ref ident) if format!("{}", ident) == lsb => {
            BitOrder::LeastSignificantFirst
        }
        _ => panic!("expected {} or {} for bit_order", msb, lsb),
    }
}
#[cfg(feature = "stable")]
fn get_bit_order(map: &mut HashMap<String, Token>) -> BitOrder {
    let msb = "MostSignificantFirst";
    let lsb = "LeastSignificantFirst";
    match map.remove("bit_order") {
        None => return BitOrder::MostSignificantFirst,
        Some(Token::Ident(ref ident)) if ident.as_ref() == msb => BitOrder::MostSignificantFirst,
        Some(Token::Ident(ref ident)) if ident.as_ref() == lsb => BitOrder::LeastSignificantFirst,
        Some(_) => panic!("expected {} or {} for bit_order", msb, lsb),
    }
}

fn check_present<T>(hash_map: &HashMap<String, T>, key: &str) {
    if !hash_map.contains_key(key) {
        panic!("{} is required", key);
    }
}

#[cfg(not(feature = "stable"))]
fn get_encoding(mut hash_map: &mut HashMap<String, TokenTree>) -> Encoding {
    check_present(&hash_map, "symbols");
    let spec = Specification {
        symbols: get_string(&mut hash_map, "symbols"),
        bit_order: get_bit_order(&mut hash_map),
        check_trailing_bits: get_bool(&mut hash_map, "check_trailing_bits").unwrap_or(true),
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
#[cfg(feature = "stable")]
fn get_encoding(mut hash_map: &mut HashMap<String, Token>) -> Encoding {
    check_present(&hash_map, "symbols");
    let spec = Specification {
        symbols: get_string(&mut hash_map, "symbols"),
        bit_order: get_bit_order(&mut hash_map),
        check_trailing_bits: get_bool(&mut hash_map, "check_trailing_bits").unwrap_or(true),
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

fn check_empty<T>(hash_map: HashMap<String, T>) {
    if !hash_map.is_empty() {
        panic!("Unexpected keys {:?}", hash_map.keys());
    }
}

#[cfg(not(feature = "stable"))]
#[proc_macro]
#[doc(hidden)]
pub fn internal_new_encoding(input: TokenStream) -> TokenStream {
    let mut hash_map = parse_map(input.into_iter());
    let encoding = get_encoding(&mut hash_map);
    check_empty(hash_map);
    format!("{:?}", encoding.internal_implementation())
        .parse()
        .unwrap()
}
#[cfg(feature = "stable")]
proc_macro_expr_impl! {
    #[doc(hidden)]
    pub fn internal_new_encoding_impl(input: &str) -> String {
        let mut hash_map = parse_map(input);
        let encoding = get_encoding(&mut hash_map);
        check_empty(hash_map);
        format!("{:?}", encoding.internal_implementation())
    }
}

#[cfg(not(feature = "stable"))]
#[proc_macro]
#[doc(hidden)]
pub fn internal_decode_array(input: TokenStream) -> TokenStream {
    let mut hash_map = parse_map(input.into_iter());
    let encoding = get_encoding(&mut hash_map);
    check_present(&mut hash_map, "name");
    let name = get_string(&mut hash_map, "name");
    check_present(&mut hash_map, "input");
    let input = get_string(&mut hash_map, "input");
    check_empty(hash_map);
    let output = encoding.decode(input.as_bytes()).unwrap();
    format!("{}: [u8; {}] = {:?};", name, output.len(), output)
        .parse()
        .unwrap()
}
#[cfg(not(feature = "stable"))]
#[proc_macro]
#[doc(hidden)]
pub fn internal_decode_slice(input: TokenStream) -> TokenStream {
    let mut hash_map = parse_map(input.into_iter());
    let encoding = get_encoding(&mut hash_map);
    check_present(&mut hash_map, "input");
    let input = get_string(&mut hash_map, "input");
    check_empty(hash_map);
    format!("{:?}", encoding.decode(input.as_bytes()).unwrap())
        .parse()
        .unwrap()
}
#[cfg(feature = "stable")]
proc_macro_expr_impl! {
    #[doc(hidden)]
    pub fn internal_decode_slice_impl(input: &str) -> String {
        let mut hash_map = parse_map(input);
        let encoding = get_encoding(&mut hash_map);
        check_present(&mut hash_map, "input");
        let input = get_string(&mut hash_map, "input");
        check_empty(hash_map);
        format!("{:?}", encoding.decode(input.as_bytes()).unwrap())
    }
}
