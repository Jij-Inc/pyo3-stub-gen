use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, Error, ItemEnum, Result, Type};

use super::{extract_documents, parse_pyo3_attrs, util::quote_option, Attr, StubType};

/// Attributes for `#[gen_stub_pyclass_enum(...)]`
#[derive(Default)]
pub(crate) struct PyEnumAttr {
    pub(crate) skip_stub_type: bool,
}

impl Parse for PyEnumAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut skip_stub_type = false;

        // Parse comma-separated key-value pairs or flags
        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;

            match key.to_string().as_str() {
                "skip_stub_type" => {
                    skip_stub_type = true;
                }
                _ => {
                    return Err(Error::new(
                        key.span(),
                        format!("Unknown parameter: {}", key),
                    ));
                }
            }

            // Check for comma separator
            if input.peek(syn::token::Comma) {
                let _: syn::token::Comma = input.parse()?;
            } else {
                break;
            }
        }

        Ok(Self { skip_stub_type })
    }
}

pub struct PyEnumInfo {
    pyclass_name: String,
    enum_type: Type,
    module: Option<String>,
    variants: Vec<(String, String)>,
    doc: String,
}

impl From<&PyEnumInfo> for StubType {
    fn from(info: &PyEnumInfo) -> Self {
        let PyEnumInfo {
            pyclass_name,
            module,
            enum_type,
            ..
        } = info;
        Self {
            ty: enum_type.clone(),
            name: pyclass_name.clone(),
            module: module.clone(),
        }
    }
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
        let mut renaming_rule = None;
        for attr in parse_pyo3_attrs(&attrs)? {
            match attr {
                Attr::Name(name) => pyclass_name = Some(name),
                Attr::Module(name) => module = Some(name),
                Attr::RenameAll(name) => renaming_rule = Some(name),
                _ => {}
            }
        }
        let struct_type = parse_quote!(#ident);
        let pyclass_name = pyclass_name.unwrap_or_else(|| ident.to_string());
        let variants = variants
            .into_iter()
            .map(|var| -> Result<(String, String)> {
                let mut var_name = None;
                for attr in parse_pyo3_attrs(&var.attrs)? {
                    if let Attr::Name(name) = attr {
                        var_name = Some(name);
                    }
                }
                let mut var_name = var_name.unwrap_or_else(|| var.ident.to_string());
                if let Some(renaming_rule) = renaming_rule {
                    var_name = renaming_rule.apply(&var_name);
                }
                let var_doc = extract_documents(&var.attrs).join("\n");
                Ok((var_name, var_doc))
            })
            .collect::<Result<Vec<(String, String)>>>()?;
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
        let variants: Vec<_> = variants
            .iter()
            .map(|(name, doc)| quote! {(#name,#doc)})
            .collect();
        tokens.append_all(quote! {
            ::pyo3_stub_gen::type_info::PyEnumInfo {
                pyclass_name: #pyclass_name,
                enum_id: std::any::TypeId::of::<#enum_type>,
                variants: &[ #(#variants),* ],
                module: #module,
                doc: #doc,
            }
        })
    }
}
