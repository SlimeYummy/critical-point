extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use std::sync::atomic::{AtomicUsize, Ordering};
use syn::*;

static COUNTER: AtomicUsize = AtomicUsize::new(1);

#[proc_macro_derive(StateDataSuper, attributes(class_id))]
pub fn state_data1(input: TokenStream) -> TokenStream {
    let class = parse_macro_input!(input as ItemStruct);
    let data = &class.ident;
    let class_id = extract_class_id(&class.attrs, "StateDataSuper");

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

#[proc_macro_derive(StateOwnerSuper, attributes(class_id))]
pub fn state_owner1(input: TokenStream) -> TokenStream {
    let class = parse_macro_input!(input as ItemStruct);
    let owner = &class.ident;
    let class_id = extract_class_id(&class.attrs, "StateOwnerSuper");
    let data_bind: Vec<_> = state_refs_to_token(&class.fields, |field| {
        return quote! { crate::state::state_binder_register(&self.#field)?; };
    });
    let data_drop: Vec<_> = state_refs_to_token(&class.fields, |field| {
        return quote! { crate::state::state_binder_unregister(&owner.#field); };
    });

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

            fn bind_state(&mut self) -> Result<(), failure::Error> {
                if self.dropper.is_bind() {
                    return Err(failure::format_err!("StateOwner::bind_state() multi times."));
                }
                #(#data_bind)*
                self.dropper.bind(self);
            }

            fn drop_state(&mut self) {
                #(#data_drop)*
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

#[proc_macro_attribute]
pub fn state_data(args: TokenStream, input: TokenStream) -> TokenStream {
    let meta = parse_macro_input!(args as AttributeArgs);
    let type_id = extract_type_id(meta, "state_data");

    let class = parse_macro_input!(input as ItemStruct);
    if !class.generics.params.is_empty() {
        panic!("#[state_data(TYPE_ID)] => Do not support generic.");
    }
    let ItemStruct {
        ident: data, vis, ..
    } = &class;
    let fields: Vec<_> = fields_to_token(&class.fields);
    let attrs: Vec<_> = attrs_to_token(&class.attrs);

    let num = COUNTER.fetch_add(1, Ordering::Relaxed);
    let sup = Ident::new(&format!("{}{}", "StateDataSuper", num), Span::call_site());

    return TokenStream::from(quote! {
        #(#attrs)*
        #vis struct #data {
            sup: #sup,
            #(#fields),*
        }

        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        struct #sup(crate::sup::StateDataSuperField);

        impl crate::sup::StateDataStatic for #data {
            fn id() -> crate::id::ClassID { return #type_id; }
        }

        impl crate::sup::StateDataSuper for #data {
            fn _obj_id(&self) -> crate::id::ObjID { return self.sup.0.obj_id; }
            fn _type_id(&self) -> crate::id::ClassID { return self.sup.0.type_id; }
            fn _lifecycle(&self) -> crate::state::StateLifecycle { return self.sup.0.lifecycle; }
        }

        impl #data {
            fn default_super() -> #sup { return #sup::default(); }

            fn new_super(obj_id: crate::id::ObjID, lifecycle: crate::state::StateLifecycle) -> #sup {
                return #sup(crate::sup::StateDataSuperField{
                    obj_id,
                    type_id: #type_id,
                    lifecycle,
                });
            }
        }
    });
}

#[proc_macro_attribute]
pub fn state_owner(args: TokenStream, input: TokenStream) -> TokenStream {
    let meta = parse_macro_input!(args as AttributeArgs);
    let type_id = extract_type_id(meta, "state_owner");

    let class = parse_macro_input!(input as ItemStruct);
    if !class.generics.params.is_empty() {
        panic!("#[state_owner(TYPE_ID)] => Do not support generic.");
    }
    if !is_derive_godot(&class.attrs) {
        panic!("#[state_owner(TYPE_ID)] => Must derive godot-plugin::NativeClass.");
    }

    let ItemStruct {
        ident: owner, vis, ..
    } = &class;
    let fields: Vec<_> = fields_to_token(&class.fields);
    let attrs: Vec<_> = attrs_to_token(&class.attrs);

    let data_bind: Vec<_> = state_refs_to_token(&class.fields, |field| {
        return quote! { crate::state::state_binder_register(&self.#field)?; };
    });
    let data_drop: Vec<_> = state_refs_to_token(&class.fields, |field| {
        return quote! { crate::state::state_binder_unregister(&owner.#field); };
    });

    let num = COUNTER.fetch_add(1, Ordering::Relaxed);
    let sup = Ident::new(&format!("{}{}", "StateOwnerSuper", num), Span::call_site());

    return TokenStream::from(quote! {
        #(#attrs)*
        #vis struct #owner {
            sup: #sup,
            #(#fields),*
        }

        #[derive(Clone, Debug, Default, PartialEq)]
        struct #sup(crate::sup::StateOwnerSuperField);

        impl #owner {
            fn default_super() -> #sup { return #sup::default(); }

            fn new_super(obj_id: crate::id::ObjID) -> #sup {
                return #sup(crate::sup::StateOwnerSuperField{
                    obj_id,
                    type_id: #type_id,
                    .. Default::default()
                });
            }
        }

        impl crate::sup::StateOwnerStatic for #owner {
            fn id() -> crate::id::ClassID { return #type_id; }
        }

        impl crate::sup::StateOwnerSuper for #owner {
            fn _obj_id(&self) -> crate::id::ObjID { return self.sup.0.obj_id; }
            fn _type_id(&self) -> crate::id::ClassID { return self.sup.0.type_id; }

            fn _bind_state(&mut self) -> Result<(), failure::Error> {
                if !self.sup.0.once {
                    #(#data_bind)*
                    self.sup.0.once = true;
                    let owner_ptr = (self as *mut #owner) as i32;
                    let header_ptr = (&self.sup as *const #sup) as i32;
                    self.sup.0.offset = owner_ptr - header_ptr;
                }
                return Ok(());
            }
        }

        impl Drop for #sup {
            fn drop(&mut self) {
                if self.0.once {
                    let hidden_ptr = (self as *mut #sup) as *mut u8;
                    let owner_ptr = unsafe { hidden_ptr.offset(self.0.offset as isize) };
                    let owner = unsafe { &*(owner_ptr as *mut #owner) };
                    #(#data_drop)*
                }
            }
        }
    });
}

#[proc_macro_attribute]
pub fn logic_obj(args: TokenStream, input: TokenStream) -> TokenStream {
    let meta = parse_macro_input!(args as AttributeArgs);
    if meta.len() != 1 {
        panic!("#[logic_obj(TYPE_ID)] => Need a ClassID.");
    }
    let type_id: TokenStream2;
    if let NestedMeta::Meta(Meta::Path(path)) = &meta[0] {
        type_id = path.into_token_stream();
    } else {
        panic!("#[logic_obj(TYPE_ID)] => Invaild ClassID.");
    }

    let class = parse_macro_input!(input as ItemStruct);
    if !class.generics.params.is_empty() {
        panic!("#[logic_obj(TYPE_ID)] => Do not support generic.");
    }
    let ItemStruct {
        ident: logic, vis, ..
    } = &class;
    let fields: Vec<_> = fields_to_token(&class.fields);
    let attrs: Vec<_> = attrs_to_token(&class.attrs);

    let num = COUNTER.fetch_add(1, Ordering::Relaxed);
    let sup = Ident::new(&format!("{}{}", "LogicObjSuper", num), Span::call_site());

    return TokenStream::from(quote! {
        #(#attrs)*
        #vis struct #logic {
            sup: #sup,
            #(#fields),*
        }

        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        struct #sup(crate::sup::LogicObjSuperField);

        impl crate::sup::LogicObjStatic for #logic {
            fn id() -> crate::id::ClassID { return #type_id; }
        }

        impl crate::sup::LogicObjSuper for #logic {
            fn _obj_id(&self) -> crate::id::ObjID { return self.sup.0.obj_id; }
            fn _type_id(&self) -> crate::id::ClassID { return self.sup.0.type_id; }
        }

        impl #logic {
            fn default_super() -> #sup { return #sup::default(); }

            fn new_super(obj_id: crate::id::ObjID) -> #sup {
                return #sup(crate::sup::LogicObjSuperField{
                    obj_id,
                    type_id: #type_id,
                });
            }
        }
    });
}

fn extract_type_id(meta: AttributeArgs, msg: &str) -> TokenStream2 {
    if meta.len() != 1 {
        panic!("#[{}(TYPE_ID)] => Need a ClassID.", msg);
    }
    if let NestedMeta::Meta(Meta::Path(path)) = &meta[0] {
        return path.into_token_stream();
    } else {
        panic!("#[{}(TYPE_ID)] => Invaild ClassID.", msg);
    }
}

fn fields_to_token(fields: &Fields) -> Vec<TokenStream2> {
    return fields
        .iter()
        .map(|field| field.into_token_stream())
        .collect();
}

fn attrs_to_token(attrs: &[Attribute]) -> Vec<TokenStream2> {
    return attrs.iter().map(|attr| attr.into_token_stream()).collect();
}

fn is_derive_godot(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if !attr.path.is_ident("derive") {
            continue;
        }
        if attr.tokens.to_string().find("NativeClass").is_some() {
            return true;
        }
    }
    return false;
}

fn state_refs_to_token<F>(fields: &Fields, func: F) -> Vec<TokenStream2>
where
    F: FnMut(Ident) -> TokenStream2,
{
    return fields
        .iter()
        .filter(|field| {
            if let Type::Path(path) = &field.ty {
                path.path.segments[0].ident.to_string() == "StateRef"
            } else {
                false
            }
        })
        .map(|field| field.ident.clone().unwrap())
        .map(func)
        .collect();
}
