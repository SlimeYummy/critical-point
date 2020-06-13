extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::*;

#[proc_macro_attribute]
pub fn state_data(args: TokenStream, input: TokenStream) -> TokenStream {
    let meta = parse_macro_input!(args as AttributeArgs);
    if meta.len() != 1 {
        panic!("#[state_data(TYPE_ID)] => Need a TypeID.");
    }
    let type_id: TokenStream2;
    if let NestedMeta::Meta(Meta::Path(path)) = &meta[0] {
        type_id = path.into_token_stream();
    } else {
        panic!("#[state_data(TYPE_ID)] => Invaild TypeID.");
    }

    let class = parse_macro_input!(input as ItemStruct);
    if !class.generics.params.is_empty() {
        panic!("#[state_data(TYPE_ID)] => Do not support generic.");
    }
    let ItemStruct{ ident: data, vis, ..} = &class;
    let fields: Vec<_> = fields_to_token(&class.fields);
    let attrs: Vec<_> = attrs_to_token(&class.attrs);

    return TokenStream::from(quote! {
        #(#attrs)*
        #vis struct #data {
            _header_: crate::state::StateHeader,
            #(#fields),*
        }

        impl crate::state::StateDataStatic for #data {
            fn id() -> crate::id::TypeID { return #type_id; }
        }

        impl crate::state::StateData for #data {
            fn obj_id(&self) -> crate::id::ObjID { return self._header_.obj_id; }
            fn type_id(&self) -> crate::id::TypeID { return self._header_.type_id; }
            fn lifecycle(&self) -> crate::state::StateLifecycle { return self._header_.lifecycle; }
        }
    });
}

#[proc_macro_attribute]
pub fn state_owner(args: TokenStream, input: TokenStream) -> TokenStream {
    let meta = parse_macro_input!(args as AttributeArgs);
    if meta.len() != 2 {
        panic!("#[state_owner(TYPE_ID, StateBinder)] => Need a TypeID & a StateBinder.");
    }

    let type_id: TokenStream2;
    if let NestedMeta::Meta(Meta::Path(path)) = &meta[0] {
        type_id = path.into_token_stream();
    } else {
        panic!("#[state_owner(TYPE_ID, StateBinder)] => Invaild TypeID.");
    }

    let binder: TokenStream2;
    if let NestedMeta::Meta(Meta::List(list)) = &meta[1] {
        binder = list.into_token_stream();
    } else {
        panic!("#[state_owner(TYPE_ID, StateBinder)] => Invaild StateBinder.");
    }

    let class = parse_macro_input!(input as ItemStruct);
    let owner = &class.ident;
    let vis = class.vis;
    let fields: Vec<_> = fields_to_token(&class.fields);
    let attrs: Vec<_> = attrs_to_token(&class.attrs);
    
    if !is_derive_godot(&class.attrs) {
        panic!("#[state_owner(TYPE_ID, StateBinder)] => Must derive godot::NativeClass.");
    }
    let helper = Ident::new("Helper", Span::call_site());
    let data_bind: Vec<_> = state_refs_to_token(&class.fields, |field| {
        return quote! { #binder.register(&self.#field) };
    });
    let data_drop: Vec<_> = state_refs_to_token(&class.fields, |field| {
        return quote! { #binder.unregister(&owner.#field) };
    });

    return TokenStream::from(quote! {
        #(#attrs)*
        #vis struct #owner {
            _helper_: #helper,
            #(#fields),*
        }

        impl #owner {
            fn bind_state(&mut self) {
                if !self._helper_.once {
                    #(#data_bind);*
                    self._helper_.once = true;
                }
            }
        }

        #[derive(Clone, Debug, Default, Hash, PartialEq)]
        struct #helper {
            once: bool,
        }

        impl Drop for #helper {
            fn drop(&mut self) {
                if self.once {
                    let owner = unsafe { &*((self as *mut #helper) as *mut #owner) };
                    #(#data_drop);*
                }
            }
        }
    });
}

fn fields_to_token(fields: &Fields) -> Vec<TokenStream2> {
    return fields.iter()
        .map(|field| field.into_token_stream())
        .collect();
}

fn attrs_to_token(attrs: &[Attribute]) -> Vec<TokenStream2> {
    return attrs.iter()
        .map(|attr| attr.into_token_stream())
        .collect();
}

fn is_derive_godot(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if !attr.path.is_ident("derive") {
            continue
        }
        if attr.tokens.to_string().find("NativeClass").is_some() {
            return true;
        }
    }
    return false;
}

fn state_refs_to_token<F>(fields: &Fields, func: F) -> Vec<TokenStream2>
where F: FnMut(&Ident) -> TokenStream2
{
    return fields.iter()
        .filter(|field| {
            println!("{:?}", field.ty);
            return field.ident.is_some();
        })
        .map(|field| field.ident.unwrap().clone())
        .map(func)
        .collect();
}
