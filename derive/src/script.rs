use super::utils::has_attr;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::*;

pub fn script_opt_impl(input: TokenStream) -> TokenStream {
    let script = parse_macro_input!(input as ItemEnum);
    let name = &script.ident;

    let mut metas = Vec::new();
    for variant in &script.variants {
        let ident = &variant.ident;
        let name = ident.to_string().to_lowercase();

        for attr in &variant.attrs {
            let token = &attr.tokens;
            if attr.path.is_ident("sign") {
                metas.push(quote! {
                    _ScriptOptMeta_::sign(ScriptOpt::#ident, #token)
                });
            } else if attr.path.is_ident("func") {
                metas.push(quote! {
                    _ScriptOptMeta_::func(ScriptOpt::#ident, #name, #token)
                });
            }
        }
    }

    return TokenStream::from(quote! {
        pub struct _ScriptOptMeta_ {
            pub op: ScriptOpt,
            pub func: bool,
            pub ident: &'static str,
            pub args: u8,
        }

        impl _ScriptOptMeta_ {
            const fn sign(
                op: ScriptOpt,
                args: u8,
            ) -> _ScriptOptMeta_ {
                return _ScriptOptMeta_{ op, func: false, ident: "", args };
            }

            const fn func(
                op: ScriptOpt,
                ident: &'static str,
                args: u8,
            ) -> _ScriptOptMeta_ {
                return _ScriptOptMeta_{ op, func: true, ident, args };
            }
        }

        const _SCRIPT_OPT_ARRAY_: &'static [_ScriptOptMeta_] = &[
            #(#metas,)*
        ];

        impl #name {
            #[inline]
            pub fn tag(&self) -> u32 {
                return *self as u32;
            }

            #[inline]
            pub fn is_sign(&self) -> bool {
                return !_SCRIPT_OPT_ARRAY_[self.tag() as usize].func;
            }

            #[inline]
            pub fn is_func(&self) -> bool {
                return _SCRIPT_OPT_ARRAY_[self.tag() as usize].func;
            }

            #[inline]
            pub fn ident(&self) -> &'static str {
                return _SCRIPT_OPT_ARRAY_[self.tag() as usize].ident;
            }

            #[inline]
            pub fn args(&self) -> usize {
                return _SCRIPT_OPT_ARRAY_[self.tag() as usize].args as usize;
            }
        }
    });
}

pub fn script_var_impl(input: TokenStream) -> TokenStream {
    let var = parse_macro_input!(input as ItemStruct);
    let type_name = &var.ident;
    let lazy_name = format_ident!("_{}Lazy_", type_name);

    let mut map_items = Vec::new();
    for field in &var.fields {
        if has_attr(&field.attrs, "script") {
            let ident = field.ident.clone().unwrap();
            let ident_str = ident.to_string();
            map_items.push(quote! {
                let name = format!("{}.{}", #type_name::prefix(), #ident_str);
                let ptr: *const #type_name = std::ptr::null();
                let offset = unsafe { &(*ptr).#ident as *const _ as usize };
                map.insert(name, (offset / std::mem::size_of::<Fx>()) as u16);
            });
        }
    }

    return TokenStream::from(quote! {
        static #lazy_name: std::lazy::SyncLazy<std::collections::HashMap<String, u16>> =
            std::lazy::SyncLazy::new(|| {
                let mut map = std::collections::HashMap::new();
                #(#map_items)*
                return map;
            });

        impl #type_name {
            pub fn fields() -> &'static std::lazy::SyncLazy<std::collections::HashMap<String, u16>> {
                return &#lazy_name;
            }

            pub fn max_offset() -> u16 {
                let ptr: *const #type_name = std::ptr::null();
                let ptr2 = unsafe { ptr.offset(1) };
                let offset = (ptr2 as usize) - (ptr as usize);
                return (offset / std::mem::size_of::<Fx>()) as u16;
            }
        }
    });
}
