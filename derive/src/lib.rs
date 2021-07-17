#![feature(once_cell)]

extern crate darling;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

mod csharp;
mod script;
mod utils;

use csharp::{csharp_enum, csharp_prop, csharp_state};
use proc_macro::TokenStream;
use quote::quote;
use script::{script_ctx_impl, script_var_impl};
use syn::*;

#[proc_macro_attribute]
pub fn def_struct(_: TokenStream, token: TokenStream) -> TokenStream {
    return token;
}

#[proc_macro_attribute]
pub fn def_enum(_: TokenStream, token: TokenStream) -> TokenStream {
    let token_copy = token.clone();
    let item_enum = parse_macro_input!(token as ItemEnum);
    csharp_enum(item_enum);
    return token_copy;
}

#[proc_macro_attribute]
pub fn def_res(attr: TokenStream, input: TokenStream) -> TokenStream {
    let class = parse_macro_input!(input as ItemStruct);
    let res = &class.ident;
    let class_id = parse_macro_input!(attr as Expr);

    return TokenStream::from(quote! {
        #class

        impl crate::resource::ResObjStatic for #res {
            #[inline]
            fn id() -> crate::id::ClassID {
                return #class_id;
            }
        }

        impl crate::resource::ResObjSuper for #res {
            #[inline]
            fn class_id(&self) -> crate::id::ClassID {
                return #class_id;
            }

            #[inline]
            fn res_id(&self) -> &crate::id::ResID {
                return &self.res_id;
            }

            #[inline]
            fn fres_id(&self) -> FastResID {
                return self.fres_id;
            }
        }
    });
}

#[proc_macro_attribute]
pub fn def_prop(attr: TokenStream, class: TokenStream) -> TokenStream {
    let class = parse_macro_input!(class as ItemStruct);
    csharp_prop(class.clone(), &attr.to_string());

    let class_name = &class.ident;
    let class_id = parse_macro_input!(attr as Expr);

    return TokenStream::from(quote! {
        #[repr(C)]
        #class

        impl crate::engine::LogicPropStatic for #class_name {
            #[inline]
            fn id() -> crate::id::ClassID {
                return #class_id;
            }
        }
    });
}

#[proc_macro_attribute]
pub fn def_state(attr: TokenStream, class: TokenStream) -> TokenStream {
    let class = parse_macro_input!(class as ItemStruct);
    csharp_state(class.clone(), &attr.to_string());

    let class_name = &class.ident;
    let class_id = parse_macro_input!(attr as Expr);

    return TokenStream::from(quote! {
        #[repr(C)]
        #class

        impl crate::engine::LogicStateStatic for #class_name {
            #[inline]
            fn id() -> crate::id::ClassID {
                return #class_id;
            }
        }
    });
}

#[proc_macro_attribute]
pub fn def_obj(attr: TokenStream, class: TokenStream) -> TokenStream {
    let class = parse_macro_input!(class as ItemStruct);
    let class_name = &class.ident;
    let class_id = parse_macro_input!(attr as Expr);

    return TokenStream::from(quote! {
        #class

        impl crate::engine::LogicObjStatic for #class_name {
            #[inline]
            fn id() -> crate::id::ClassID {
                return #class_id;
            }
        }

        impl crate::engine::LogicObjSuper for #class_name {
            #[inline]
            fn class_id(&self) -> crate::id::ClassID {
                return #class_id;
            }

            #[inline]
            fn obj_id(&self) -> crate::id::ObjID {
                return self.obj_id;
            }
        }
    });
}

#[proc_macro_attribute]
pub fn script_var(attr: TokenStream, body: TokenStream) -> TokenStream {
    return script_var_impl(attr, body);
}

#[proc_macro_attribute]
pub fn script_ctx(attr: TokenStream, body: TokenStream) -> TokenStream {
    return script_ctx_impl(attr, body);
}
