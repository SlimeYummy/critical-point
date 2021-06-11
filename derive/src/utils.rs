#![allow(dead_code)]

use syn::*;

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

pub fn has_attr(attrs: &[Attribute], name: &str) -> bool {
    return attrs.iter().find(|attr| attr.path.is_ident(name)).is_some();
}
