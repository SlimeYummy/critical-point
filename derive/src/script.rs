use super::utils::{find_attr, IDGener};
use darling::FromMeta;
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::*;

static VAR_ID_GENER: IDGener = IDGener::new();
static CTX_ID_GENER: IDGener = IDGener::new();

#[derive(Debug, FromMeta)]
struct VarAttrs {
    prefix: String,
}

pub fn script_var_impl(attr_token: TokenStream, var_token: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(attr_token as AttributeArgs);
    let VarAttrs { prefix } = VarAttrs::from_list(&attr_args).unwrap();

    let mut var_ast = parse_macro_input!(var_token as ItemStruct);
    let var_type = &var_ast.ident;
    let var_id = VAR_ID_GENER.gen();
    let fields_ident = format_ident!("_VAR_FIELDS_{}_", var_id);

    let mut fields_tokens = Vec::new();
    for field in &mut var_ast.fields {
        if let Some((idx, _)) = find_attr(&field.attrs, "script_skip") {
            field.attrs.remove(idx);
            continue;
        }

        if (&field.ty).into_token_stream().to_string() != "Fx" {
            continue;
        }

        let field_ident = field.ident.clone().unwrap();
        let script_str = format!("{}.{}", prefix, field_ident);
        fields_tokens.push(quote! {
            let ptr: *const #var_type = std::ptr::null();
            let offset = unsafe { &(*ptr).#field_ident as *const _ as usize };
            map.insert(#script_str, crate::script::ScriptVarField{
                ident: #script_str,
                offset: (offset / std::mem::size_of::<Fx>()) as u16,
            });
        });
    }

    return TokenStream::from(quote! {
        #var_ast

        static #fields_ident: crate::script::ScriptVarFields = std::lazy::SyncLazy::new(|| {
            let mut map = std::collections::HashMap::new();
            #(#fields_tokens)*
            return map;
        });

        impl crate::script::ScriptVar for #var_type {
            fn var_id() -> u8 { return #var_id; }

            fn prefix() -> &'static str { return #prefix; }

            fn fields() -> &'static crate::script::ScriptVarFields { return &#fields_ident; }

            fn max_offset() -> u16 {
                let ptr: *const #var_type = std::ptr::null();
                let ptr2 = unsafe { ptr.offset(1) };
                let offset = (ptr2 as usize) - (ptr as usize);
                return (offset / std::mem::size_of::<Fx>()) as u16;
            }
        }
    });
}

pub fn script_ctx_impl(_: TokenStream, struct_token: TokenStream) -> TokenStream {
    let ctx_ast = parse_macro_input!(struct_token as ItemStruct);
    let ctx_type = &ctx_ast.ident;
    let ctx_id = CTX_ID_GENER.gen();
    let fields_ident = format_ident!("_CTX_FIELDS_{}_", ctx_id);
    let vars_ident = format_ident!("_CTX_VARS_{}_", ctx_id);

    let fields_len = ctx_ast.fields.len();
    if fields_len > 14 {
        panic!("Too many fields")
    }

    let mut fields_tokens = Vec::new();
    let mut vars_tokens = Vec::new();
    for (idx, field) in ctx_ast.fields.iter().enumerate() {
        let typ = extract_type_reference(&field.ty);
        let field_type = &typ.elem;
        let writable = typ.mutability.is_some();

        fields_tokens.push(quote! {
            for (ident, item) in #field_type::fields().iter() {
                let segment = crate::script::SEGMENT_VARS_START + (#idx as u8);
                map.insert(*ident, crate::script::ScriptCtxField{
                    ident,
                    writable: #writable,
                    addr: crate::script::ScriptAddr::new(segment, item.offset),
                });
            }
        });

        vars_tokens.push(quote! {
            map.insert(#field_type::var_id(), crate::script::ScriptCtxVar{
                var_id: #field_type::var_id(),
                segment: crate::script::SEGMENT_VARS_START + (#idx as u8),
                writable: #writable,
            });
        });
    }

    return TokenStream::from(quote! {
        #ctx_ast

        static #fields_ident: crate::script::ScriptCtxFields = std::lazy::SyncLazy::new(|| {
            use crate::script::ScriptVar;

            let mut map = std::collections::HashMap::new();
            #(#fields_tokens)*
            return map;
        });

        static #vars_ident: crate::script::ScriptCtxVars = std::lazy::SyncLazy::new(|| {
            use crate::script::ScriptVar;

            let mut map = std::collections::HashMap::new();
            #(#vars_tokens)*
            return map;
        });

        impl<'t> crate::script::ScriptCtx for #ctx_type<'t> {
            fn ctx_id() -> u8 { return #ctx_id; }

            fn fields() -> &'static crate::script::ScriptCtxFields { return &#fields_ident; }

            fn vars() -> &'static crate::script::ScriptCtxVars { return &#vars_ident; }

            fn fill_segments(&self, segments: &mut [*mut crate::script::ScriptVal]) {
                let ptrs: [*mut crate::script::ScriptVal; #fields_len] = unsafe { std::mem::transmute_copy(self) };
                segments[..#fields_len].copy_from_slice(&ptrs);
            }
        }
    });
}

fn extract_type_reference<'t>(typ: &'t Type) -> &'t TypeReference {
    match typ {
        Type::Reference(refer) => return refer,
        _ => panic!("extract_type_reference()"),
    };
}
