use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse_quote, Error, ItemEnum, Result, Type};

use super::{extract_documents, parse_pyo3_attrs, util::quote_option, Attr, PyClassAttr, StubType};

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

impl PyEnumInfo {
    /// Create PyEnumInfo from ItemEnum with PyClassAttr for module override support
    pub fn from_item_with_attr(
        ItemEnum {
            variants,
            attrs,
            ident,
            ..
        }: ItemEnum,
        attr: &PyClassAttr,
    ) -> Result<Self> {
        let doc = extract_documents(&attrs).join("\n");
        let mut pyclass_name = None;
        let mut pyo3_module = None;
        let mut gen_stub_standalone_module = None;
        let mut renaming_rule = None;
        for attr_item in parse_pyo3_attrs(&attrs)? {
            match attr_item {
                Attr::Name(name) => pyclass_name = Some(name),
                Attr::Module(name) => pyo3_module = Some(name),
                Attr::GenStubModule(name) => gen_stub_standalone_module = Some(name),
                Attr::RenameAll(name) => renaming_rule = Some(name),
                _ => {}
            }
        }

        // Validate: inline and standalone gen_stub modules must not conflict
        if let (Some(inline_mod), Some(standalone_mod)) =
            (&attr.module, &gen_stub_standalone_module)
        {
            if inline_mod != standalone_mod {
                return Err(Error::new(
                    ident.span(),
                    format!(
                        "Conflicting module specifications: #[gen_stub_pyclass_enum(module = \"{}\")] \
                         and #[gen_stub(module = \"{}\")]. Please use only one.",
                        inline_mod, standalone_mod
                    ),
                ));
            }
        }

        // Priority: inline > standalone > pyo3 > default
        let module = if let Some(inline_mod) = &attr.module {
            Some(inline_mod.clone()) // Priority 1: #[gen_stub_pyclass_enum(module = "...")]
        } else if let Some(standalone_mod) = gen_stub_standalone_module {
            Some(standalone_mod) // Priority 2: #[gen_stub(module = "...")]
        } else {
            pyo3_module // Priority 3: #[pyo3(module = "...")]
        };

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

impl TryFrom<ItemEnum> for PyEnumInfo {
    type Error = Error;
    fn try_from(item: ItemEnum) -> Result<Self> {
        // Use the new method with default PyClassAttr
        Self::from_item_with_attr(item, &PyClassAttr::default())
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
