use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse_quote, Error, Fields, Result, Type, Variant};
use syn::spanned::Spanned;
use crate::gen_stub::renaming::RenamingRule;
use super::{extract_documents, parse_pyo3_attrs, util::quote_option, Attr, MemberInfo, StubType};

pub struct PyClassTreeInfo {
    pyclass_name: String,
    struct_type: Type,
    module: Option<String>,
    members: Vec<MemberInfo>,
    doc: String,
    bases: Vec<Type>,
    classes: Vec<PyClassTreeInfo>
}

impl From<&PyClassTreeInfo> for StubType {
    fn from(info: &PyClassTreeInfo) -> Self {
        let PyClassTreeInfo {
            pyclass_name,
            module,
            struct_type,
            ..
        } = info;
        Self {
            ty: struct_type.clone(),
            name: pyclass_name.clone(),
            module: module.clone(),
        }
    }
}

impl PyClassTreeInfo {
    pub fn from_variant(variant: Variant, enum_: Ident, renaming_rule: &Option<RenamingRule>) -> Result<Self> {
        let Variant {
            ident,
            fields,
            attrs,
            ..
        } = variant;

        let struct_type: Type = parse_quote!(#enum_);
        let mut pyclass_name = None;
        let mut module = None;
        let mut bases = Vec::new();
        for attr in parse_pyo3_attrs(&attrs)? {
            match attr {
                Attr::Name(name) => pyclass_name = Some(name),
                Attr::Module(name) => {
                    module = Some(name);
                }
                Attr::Extends(typ) => bases.push(typ),
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
            Fields::Unnamed(_) => {
                return Err(Error::new(fields.span(), "Unnamed fields are not supported for PyClassInfo"))
            }
        }


        let doc = extract_documents(&attrs).join("\n");
        Ok(Self {
            struct_type,
            pyclass_name,
            members,
            module,
            doc,
            bases,
            classes: Vec::new(),
        })
    }
}

impl ToTokens for PyClassTreeInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            pyclass_name,
            struct_type,
            members,
            doc,
            module,
            bases,
            classes,
        } = self;
        let module = quote_option(module);
        tokens.append_all(quote! {
            ::pyo3_stub_gen::type_info::PyClassTreeInfo {
                pyclass_name: #pyclass_name,
                struct_id: std::any::TypeId::of::<#struct_type>,
                members: &[ #( #members),* ],
                classes: &[ #( #classes ),*],
                module: #module,
                doc: #doc,
                bases: &[ #( <#bases as ::pyo3_stub_gen::PyStubType>::type_output ),* ],
            }
        })
    }
}

