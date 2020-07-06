extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::*;

#[proc_macro_derive(StateDataX, attributes(class_id))]
pub fn state_data1(input: TokenStream) -> TokenStream {
    let class = parse_macro_input!(input as ItemStruct);
    let data = &class.ident;
    let class_id = extract_class_id(&class.attrs, "StateDataX");

    return TokenStream::from(quote! {
        impl crate::state::StateDataStatic for #data {
            fn id() -> crate::id::ClassID {
                return #class_id;
            }

            fn init(obj_id: crate::id::ObjID, lifecycle: crate::state::StateLifecycle) -> Self {
                let mut this = Self::default();
                this.obj_id = obj_id;
                this.lifecycle = lifecycle;
                return this;
            }
        }

        impl crate::state::StateData for #data {
            fn class_id(&self) -> crate::id::ClassID {
                return #class_id;
            }

            fn obj_id(&self) -> crate::id::ObjID {
                return self.obj_id;
            }

            fn lifecycle(&self) -> crate::state::StateLifecycle {
                return self.lifecycle;
            }
        }
    });
}

#[proc_macro_derive(StateOwnerX, attributes(class_id))]
pub fn state_owner1(input: TokenStream) -> TokenStream {
    let class = parse_macro_input!(input as ItemStruct);
    let owner = &class.ident;
    let class_id = extract_class_id(&class.attrs, "StateOwnerX");

    return TokenStream::from(quote! {
        impl crate::state::StateOwnerStatic for #owner {
            fn id() -> crate::id::ClassID {
                return #class_id;
            }
        }

        impl crate::state::StateOwner for #owner {
            fn class_id(&self) -> crate::id::ClassID {
                return #class_id;
            }

            fn obj_id(&self) -> crate::id::ObjID {
                return self.obj_id;
            }
        }
    });
}

#[proc_macro_derive(LogicObjX, attributes(class_id))]
pub fn logic_obj1(input: TokenStream) -> TokenStream {
    let class = parse_macro_input!(input as ItemStruct);
    let logic = &class.ident;
    let class_id = extract_class_id(&class.attrs, "LogicObjX");

    return TokenStream::from(quote! {
        impl crate::logic::LogicObjStatic for #logic {
            fn id() -> crate::id::ClassID {
                return #class_id;
            }
        }

        impl crate::logic::LogicObjSuper for #logic {
            fn class_id(&self) -> crate::id::ClassID {
                return #class_id;
            }

            fn obj_id(&self) -> crate::id::ObjID {
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
