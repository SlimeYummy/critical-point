#![allow(dead_code)]

use darling::FromMeta;
use proc_macro::TokenStream;
use std::sync::atomic::{AtomicU8, Ordering};
use syn::*;

pub struct IDGener {
    pub counter: AtomicU8,
}

impl IDGener {
    pub const fn new() -> IDGener {
        return IDGener {
            counter: AtomicU8::new(0),
        };
    }

    pub fn gen(&self) -> u8 {
        let var_id = self.counter.fetch_add(1, Ordering::Relaxed);
        if var_id == u8::MAX {
            panic!("IDGener::gen()");
        }
        return var_id;
    }
}

pub fn extract_attr_raw(attrs: &[Attribute], name: &str) -> String {
    return attrs
        .iter()
        .find(|attr| attr.path.is_ident(name))
        .map(|attr| attr.tokens.to_string())
        .unwrap_or(String::new());
}

pub fn extract_attr_expr(attrs: &[Attribute], name: &str) -> Expr {
    return attrs
        .iter()
        .find(|attr| attr.path.is_ident(name))
        .map(|attr| attr.parse_args::<Expr>().unwrap())
        .expect(&format!("Need a #[{}(...)].", name));
}

pub fn extract_attr_type(attrs: &[Attribute], name: &str) -> Type {
    return attrs
        .iter()
        .find(|attr| attr.path.is_ident(name))
        .map(|attr| attr.parse_args::<Type>().unwrap())
        .expect(&format!("Need a #[{}(...)].", name));
}

pub fn find_attr<'t>(attrs: &'t [Attribute], name: &str) -> Option<(usize, &'t Attribute)> {
    return attrs
        .iter()
        .enumerate()
        .find(|(_, attr)| attr.path.is_ident(name));
}

pub fn extract_attr_darling<M: FromMeta>(attr: &Attribute) -> M {
    let attr_token: TokenStream = attr
        .tokens
        .to_string()
        .trim_matches(&['(', ')'] as &[_])
        .parse()
        .unwrap();
    let attr_args = syn::parse_macro_input::parse::<AttributeArgs>(attr_token).unwrap();
    return M::from_list(&attr_args).unwrap();
}
