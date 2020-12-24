extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::*;

#[proc_macro_derive(ResObjX, attributes(class_id))]
pub fn res_obj(input: TokenStream) -> TokenStream {
    let class = parse_macro_input!(input as ItemStruct);
    let res = &class.ident;
    let class_id = extract_class_id(&class.attrs, "ResObjX");

    return TokenStream::from(quote! {
        impl crate::resource::ResObjStatic for #res {
            #[inline]
            fn id() -> crate::id::ClassID {
                return crate::id::ClassID::#class_id;
            }
        }

        impl crate::resource::ResObjSuper for #res {
            #[inline]
            fn class_id(&self) -> crate::id::ClassID {
                return crate::id::ClassID::#class_id;
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

#[proc_macro_derive(StateDataX, attributes(class_id))]
pub fn state_data(input: TokenStream) -> TokenStream {
    let class = parse_macro_input!(input as ItemStruct);
    let data = &class.ident;
    let class_id = extract_class_id(&class.attrs, "StateDataX");

    return TokenStream::from(quote! {
        impl crate::state::StateDataStatic for #data {
            #[inline]
            fn id() -> crate::id::ClassID {
                return crate::id::ClassID::#class_id;
            }

            fn init(
                fobj_id: crate::id::FastObjID,
                lifecycle: crate::state::StateLifecycle,
            ) -> Self {
                let mut this = Self::default();
                this.fobj_id = fobj_id;
                this.lifecycle = lifecycle;
                return this;
            }
        }

        impl crate::state::StateData for #data {
            #[inline]
            fn class_id(&self) -> crate::id::ClassID {
                return crate::id::ClassID::#class_id;
            }

            #[inline]
            fn fobj_id(&self) -> crate::id::FastObjID {
                return self.fobj_id;
            }

            #[inline]
            fn lifecycle(&self) -> crate::state::StateLifecycle {
                return self.lifecycle;
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
        impl crate::engine::LogicObjStatic for #logic {
            #[inline]
            fn id() -> crate::id::ClassID {
                return crate::id::ClassID::#class_id;
            }
        }

        impl crate::engine::LogicObj for #logic {
            #[inline]
            fn class_id(&self) -> crate::id::ClassID {
                return crate::id::ClassID::#class_id;
            }

            #[inline]
            fn fobj_id(&self) -> crate::id::FastObjID {
                return self.fobj_id;
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
