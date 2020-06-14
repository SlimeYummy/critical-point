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
            header: crate::state::StateDataHeader,
            #(#fields),*
        }

        impl crate::state::StateDataStatic for #data {
            fn id() -> crate::id::TypeID { return #type_id; }
        }

        impl crate::state::StateData for #data {
            fn obj_id(&self) -> crate::id::ObjID { return self.header.obj_id; }
            fn type_id(&self) -> crate::id::TypeID { return self.header.type_id; }
            fn lifecycle(&self) -> crate::state::StateLifecycle { return self.header.lifecycle; }
        }

        impl #data {
            fn default_header() -> crate::state::StateDataHeader {
                return crate::state::StateDataHeader::default();
            }
        }
    });
}

#[proc_macro_attribute]
pub fn state_owner(args: TokenStream, input: TokenStream) -> TokenStream {
    let meta = parse_macro_input!(args as AttributeArgs);
    if meta.len() != 1 {
        panic!("#[state_owner(TYPE_ID)] => Need a TypeID.");
    }
    let type_id: TokenStream2;
    if let NestedMeta::Meta(Meta::Path(path)) = &meta[0] {
        type_id = path.into_token_stream();
    } else {
        panic!("#[state_owner(TYPE_ID)] => Invaild TypeID.");
    }

    let class = parse_macro_input!(input as ItemStruct);
    if !class.generics.params.is_empty() {
        panic!("#[state_owner(TYPE_ID)] => Do not support generic.");
    }
    if !is_derive_godot(&class.attrs) {
        panic!("#[state_owner(TYPE_ID, StateBinder)] => Must derive godot::NativeClass.");
    }

    let ItemStruct{ ident: owner, vis, ..} = &class;
    let fields: Vec<_> = fields_to_token(&class.fields);
    let attrs: Vec<_> = attrs_to_token(&class.attrs);

    let data_bind: Vec<_> = state_refs_to_token(&class.fields, |field| {
        return quote! { crate::state::state_binder_register(&self.#field)?; };
    });
    let data_drop: Vec<_> = state_refs_to_token(&class.fields, |field| {
        return quote! { crate::state::state_binder_unregister(&owner.#field); };
    });

    let header = Ident::new("StateOwnerHeader", Span::call_site());

    return TokenStream::from(quote! {
        #(#attrs)*
        #vis struct #owner {
            header: #header,
            #(#fields),*
        }

        impl crate::state::StateOwner for #owner {}

        impl crate::state::StateOwnerStatic for #owner {
            fn id() -> crate::id::TypeID { return #type_id; }
        }

        impl #owner {
            fn default_header() -> #header {
                return #header::default();
            }

            fn bind_state(&mut self) -> Result<(), failure::Error> {
                if !self.header.once {
                    #(#data_bind)*
                    self.header.once = true;
                    let owner_ptr = (self as *mut #owner) as i32;
                    let header_ptr = (&self.header as *const #header) as i32;
                    self.header.offset = owner_ptr - header_ptr;
                }
                return Ok(());
            }
        }

        #[derive(Clone, Debug, Default, Hash, PartialEq)]
        struct #header {
            once: bool,
            offset: i32,
        }

        impl Drop for #header {
            fn drop(&mut self) {
                if self.once {
                    let header_ptr = (self as *mut #header) as *mut u8;
                    let owner_ptr = unsafe { header_ptr.offset(self.offset as isize) };
                    let owner = unsafe { &*(owner_ptr as *mut #owner) };
                    #(#data_drop)*
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
where F: FnMut(Ident) -> TokenStream2
{
    return fields.iter()
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
