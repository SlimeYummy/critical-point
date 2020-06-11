extern crate proc_macro;
extern crate syn;
extern crate quote;

use proc_macro::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Data, Item, ItemStruct, parse_macro_input};

#[proc_macro_attribute]
pub fn state_data(args: TokenStream, input: TokenStream) -> TokenStream {
    let st = parse_macro_input!(input as ItemStruct);
    let name = &st.ident;
    let vis = st.vis;
    let fields: Vec<_> = st.fields.iter()
        .map(|field| field.into_token_stream())
        .collect();
    let attrs: Vec<_> = st.attrs.iter()
        .map(|attr| attr.into_token_stream())
        .collect();
    return TokenStream::from(quote! {
        #(#attrs)*
        #vis struct #name{
            pub obj_id: crate::id::ObjectID,
            pub lifecycle: crate::state_pool::StateLifecycle,
            #(#fields),*
        }
    });
}
