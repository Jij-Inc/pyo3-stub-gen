use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Fields, Result, Variant};
use syn::spanned::Spanned;
use crate::gen_stub::attr::{extract_documents, parse_pyo3_attrs, Attr};
use crate::gen_stub::member::MemberInfo;
use crate::gen_stub::renaming::RenamingRule;
use crate::gen_stub::util::quote_option;

pub struct VariantInfo {
    pyclass_name: String,
    module: Option<String>,
    fields: Vec<MemberInfo>,
    doc: String,
}


impl VariantInfo {
    pub fn from_variant(variant: Variant, renaming_rule: &Option<RenamingRule>) -> Result<Self> {
        let Variant {
            ident,
            fields,
            attrs,
            ..
        } = variant;

        let mut pyclass_name = None;
        let mut module = None;
        for attr in parse_pyo3_attrs(&attrs)? {
            match attr {
                Attr::Name(name) => pyclass_name = Some(name),
                Attr::Module(name) => {
                    module = Some(name);
                }
                _ => {}
            }
        }

        let mut pyclass_name = pyclass_name.unwrap_or_else(|| ident.to_string());
        if let Some(renaming_rule) = renaming_rule {
            pyclass_name = renaming_rule.apply(&pyclass_name);
        }

        let mut members = Vec::new();

        match fields {
            Fields::Unit => {},
            Fields::Named(fields) => {
                for field in fields.named {
                    members.push(MemberInfo::try_from(field)?)
                }
            },
            Fields::Unnamed(fields) => {
                for (i, field) in fields.unnamed.iter().enumerate() {
                    let mut named_field = field.clone();
                    named_field.ident = Some(Ident::new(&format!("_{}", i), field.ident.span()));
                    members.push(MemberInfo::try_from(named_field)?)
                }
            }
        }


        let doc = extract_documents(&attrs).join("\n");
        Ok(Self {
            pyclass_name,
            fields: members,
            module,
            doc,

        })
    }
}

impl ToTokens for VariantInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            pyclass_name,
            fields,
            doc,
            module,
        } = self;
        let module = quote_option(module);
        tokens.append_all(quote! {
            ::pyo3_stub_gen::type_info::VariantInfo {
                pyclass_name: #pyclass_name,
                fields: &[ #( #fields),* ],
                module: #module,
                doc: #doc,
            }
        })
    }
}