extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::*;

#[proc_macro_derive(StateDataX, attributes(class_id))]
pub fn state_data(input: TokenStream) -> TokenStream {
    let class = parse_macro_input!(input as ItemStruct);
    let data = &class.ident;
    let class_id = extract_class_id(&class.attrs, "StateDataX");

    return TokenStream::from(quote! {
        impl core::state::StateDataStatic for #data {
            fn id() -> core::id::ClassID {
                return #class_id;
            }

            fn init(
                obj_id: core::id::ObjID,
                lifecycle: core::state::StateLifecycle,
            ) -> Self {
                let mut this = Self::default();
                this.obj_id = obj_id;
                this.lifecycle = lifecycle;
                return this;
            }
        }

        impl core::state::StateData for #data {
            fn class_id(&self) -> core::id::ClassID {
                return #class_id;
            }

            fn obj_id(&self) -> core::id::ObjID {
                return self.obj_id;
            }

            fn lifecycle(&self) -> core::state::StateLifecycle {
                return self.lifecycle;
            }
        }
    });
}

#[proc_macro_derive(StateOwnerX, attributes(class_id))]
pub fn state_owner(input: TokenStream) -> TokenStream {
    let class = parse_macro_input!(input as ItemStruct);
    let owner = &class.ident;
    let class_id = extract_class_id(&class.attrs, "StateOwnerX");

    return TokenStream::from(quote! {
        impl core::state::StateOwnerStatic for #owner {
            fn id() -> core::id::ClassID {
                return #class_id;
            }
        }

        impl core::state::StateOwner for #owner {
            fn class_id(&self) -> core::id::ClassID {
                return #class_id;
            }

            fn obj_id(&self) -> core::id::ObjID {
                return self.obj_id;
            }
        }
    });
}

#[proc_macro_derive(LogicObjX, attributes(class_id))]
pub fn logic_obj(input: TokenStream) -> TokenStream {
    let class = parse_macro_input!(input as ItemStruct);
    let logic = &class.ident;
    let class_id = extract_class_id(&class.attrs, "LogicObjX");

    return TokenStream::from(quote! {
        impl core::logic::LogicObjStatic for #logic {
            fn id() -> core::id::ClassID {
                return #class_id;
            }
        }

        impl core::logic::LogicObjSuper for #logic {
            fn class_id(&self) -> core::id::ClassID {
                return #class_id;
            }

            fn obj_id(&self) -> core::id::ObjID {
                return self.obj_id;
            }
        }
    });
}

fn extract_class_id(attrs: &[Attribute], msg: &str) -> TokenStream2 {
    let res = attrs.iter().find(|a| a.path.is_ident("class_id"));
    if let Some(attr) = res {
        if let Ok(expr) = attr.parse_args::<Expr>() {
            return expr.into_token_stream();
        }
    }
    panic!("#[{}(CLASS_ID)] => Need a ClassID.", msg);
}
