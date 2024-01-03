use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse_quote, Error, ItemEnum, Result, Type};

use super::{extract_documents, parse_pyo3_attrs, util::quote_option, Attr};

pub struct PyEnumInfo {
    pyclass_name: String,
    enum_type: Type,
    module: Option<String>,
    variants: Vec<String>,
    doc: String,
}

impl TryFrom<ItemEnum> for PyEnumInfo {
    type Error = Error;
    fn try_from(
        ItemEnum {
            variants,
            attrs,
            ident,
            ..
        }: ItemEnum,
    ) -> Result<Self> {
        let doc = extract_documents(&attrs).join("\n");
        let mut pyclass_name = None;
        let mut module = None;
        for attr in parse_pyo3_attrs(&attrs)? {
            match attr {
                Attr::Name(name) => pyclass_name = Some(name),
                Attr::Module(name) => module = Some(name),
                _ => {}
            }
        }
        let struct_type = parse_quote!(#ident);
        let pyclass_name = pyclass_name.unwrap_or_else(|| ident.to_string());
        let variants = variants
            .into_iter()
            .map(|var| -> Result<String> {
                let mut var_name = None;
                for attr in parse_pyo3_attrs(&var.attrs)? {
                    if let Attr::Name(name) = attr {
                        var_name = Some(name);
                    }
                }
                Ok(var_name.unwrap_or_else(|| var.ident.to_string()))
            })
            .collect::<Result<Vec<String>>>()?;
        Ok(Self {
            doc,
            enum_type: struct_type,
            pyclass_name,
            module,
            variants,
        })
    }
}

impl ToTokens for PyEnumInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            pyclass_name,
            enum_type,
            variants,
            doc,
            module,
        } = self;
        let module = quote_option(module);
        tokens.append_all(quote! {
            crate::stub::PyEnumInfo {
                pyclass_name: #pyclass_name,
                enum_id: std::any::TypeId::of::<#enum_type>,
                variants: &[ #(#variants),* ],
                module: #module,
                doc: #doc,
            }
        })
    }
}
