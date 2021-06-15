use super::utils::extract_attr_raw;
use proc_macro::TokenStream;
use quote::quote;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::lazy::SyncLazy;
use std::sync::Mutex;
use syn::{ItemEnum, ItemStruct, Type};

pub struct AutoGenFile {
    file: Mutex<File>,
}

static FILE: SyncLazy<AutoGenFile> = SyncLazy::new(|| AutoGenFile::new().unwrap());

impl AutoGenFile {
    fn new() -> Result<AutoGenFile, Box<dyn Error>> {
        let mut file = File::create("./target/FFIAutoGen.cs")?;
        file.write_all(
            [
                "using System.Runtime.InteropServices;",
                "",
                "namespace CriticalPoint {",
            ]
            .join("\n")
            .as_bytes(),
        )?;

        return Ok(AutoGenFile {
            file: Mutex::new(file),
        });
    }

    fn write(&self, content: &str) -> Result<(), Box<dyn Error>> {
        let mut file = self.file.lock().unwrap();
        file.write_all(content.as_bytes())?;
        file.write_all("}\n".as_bytes())?;
        file.seek(SeekFrom::Current(-2))?;
        return Ok(());
    }

    fn write_all(&self, contents: &[String]) -> Result<(), Box<dyn Error>> {
        return self.write(&contents.join("\n"));
    }
}

pub fn csharp_enum(item_enum: ItemEnum) {
    let raw = extract_attr_raw(&item_enum.attrs, "repr");
    let cs_type = match raw.as_str() {
        "(i8)" => "sbyte",
        "(u8)" => "byte",
        "(i16)" => "short",
        "(u16)" => "ushort",
        "(i32)" => "int",
        "(u32)" => "uint",
        "(i64)" => "long",
        "(u64)" => "ulong",
        _ => panic!("Need a #[repr(i8|u8|i16|u16|i32|u32|i64|u64)]."),
    };

    let mut lines = Vec::with_capacity(item_enum.variants.len());
    lines.push("".to_string());
    lines.push(format!(
        "    public enum {}: {} {{",
        item_enum.ident, cs_type
    ));
    for variant in &item_enum.variants {
        if variant.fields.len() != 0 {
            panic!("Unsupported enum");
        }
        if let Some((_, expr)) = &variant.discriminant {
            let token = TokenStream::from(quote! { #expr });
            lines.push(format!("        {} = {},", variant.ident, token));
        } else {
            lines.push(format!("        {},", variant.ident));
        }
    }
    lines.push(format!("    }}\n"));

    FILE.write_all(&lines).unwrap();
}

pub fn csharp_prop(item_struct: ItemStruct, class_id: &str) {
    let lines = csharp_prop_or_state(item_struct, "DefProp", class_id);
    FILE.write_all(&lines).unwrap();
}

pub fn csharp_state(item_struct: ItemStruct, class_id: &str) {
    let lines = csharp_prop_or_state(item_struct, "DefState", class_id);
    FILE.write_all(&lines).unwrap();
}

fn csharp_prop_or_state(item_struct: ItemStruct, def: &str, class_id: &str) -> Vec<String> {
    let mut lines = Vec::with_capacity(item_struct.fields.len() + 2);
    lines.push(format!(""));
    lines.push(format!("    [StructLayout(LayoutKind.Sequential)]"));
    lines.push(format!(
        "    [{}({})]",
        def,
        str::replace(class_id, " :: ", ".")
    ));
    lines.push(format!("    public struct {} {{", item_struct.ident));
    let mut counter = 1;
    for field in &item_struct.fields {
        let cs_type = translate_type(&field.ty);
        if let Some(ident) = &field.ident {
            lines.push(format!("        public {} {};", cs_type, ident));
        } else {
            lines.push(format!("        public {} _{};", cs_type, counter));
            counter += 1;
        }
    }
    lines.push(format!("    }}\n"));
    return lines;
}

fn translate_type(ty: &Type) -> String {
    if let Type::Path(path) = ty {
        if let Some(ident) = path.path.get_ident() {
            let ident = ident.to_string();
            return match ident.as_str() {
                "bool" => "bool".to_string(),
                "i8" => "sbyte".to_string(),
                "u8" => "byte".to_string(),
                "i16" => "short".to_string(),
                "u16" => "ushort".to_string(),
                "i32" => "int".to_string(),
                "u32" => "uint".to_string(),
                "i64" => "long".to_string(),
                "u64" => "ulong".to_string(),
                "f32" => "float".to_string(),
                "f64" => "double".to_string(),
                _ => ident,
            };
        } else {
            let token = TokenStream::from(quote! { #path });
            return str::replace(&token.to_string(), " ", "");
        }
    }
    panic!("Unsupport type");
}
