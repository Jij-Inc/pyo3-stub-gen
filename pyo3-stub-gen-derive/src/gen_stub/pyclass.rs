use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse_quote, Error, Fields, ItemEnum, ItemStruct, Result, Type, Variant};
use syn::spanned::Spanned;
use crate::gen_stub::renaming::RenamingRule;
use crate::gen_stub::variant::VariantInfo;
use super::{extract_documents, parse_pyo3_attrs, util::quote_option, Attr, MemberInfo, StubType};

pub struct PyClassInfo {
    pyclass_name: String,
    struct_type: Type,
    module: Option<String>,
    members: Vec<MemberInfo>,
    classes: Vec<PyClassInfo>,
    doc: String,
    bases: Vec<Type>,
}

impl From<&PyClassInfo> for StubType {
    fn from(info: &PyClassInfo) -> Self {
        let PyClassInfo {
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

impl TryFrom<ItemStruct> for PyClassInfo {
    type Error = Error;
    fn try_from(item: ItemStruct) -> Result<Self> {
        let ItemStruct {
            ident,
            attrs,
            fields,
            ..
        } = item;
        let struct_type: Type = parse_quote!(#ident);
        let mut pyclass_name = None;
        let mut module = None;
        let mut is_get_all = false;
        let mut bases = Vec::new();
        for attr in parse_pyo3_attrs(&attrs)? {
            match attr {
                Attr::Name(name) => pyclass_name = Some(name),
                Attr::Module(name) => {
                    module = Some(name);
                }
                Attr::GetAll => is_get_all = true,
                Attr::Extends(typ) => bases.push(typ),
                _ => {}
            }
        }
        let pyclass_name = pyclass_name.unwrap_or_else(|| ident.to_string());
        let mut members = Vec::new();
        for field in fields {
            if is_get_all || MemberInfo::is_candidate_field(&field)? {
                members.push(MemberInfo::try_from(field)?)
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

impl PyClassInfo {
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

impl TryFrom<ItemEnum> for PyClassInfo {
    type Error = Error;

    fn try_from(item: ItemEnum) -> Result<Self> {
        let ItemEnum {
            variants,
            attrs,
            ident,

            ..
        } = item;

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
        let pyclass_name = pyclass_name.unwrap_or_else(|| ident.clone().to_string());

        let mut members = Vec::new();
        let mut classes = Vec::new();
        for variant in variants {
            let variant = VariantInfo::from_variant(variant, ident.clone(), &renaming_rule)?;
            match variant {
                VariantInfo::Tuple {tuple} => members.push(tuple),
                VariantInfo::Struct {pyclass} => {
                    classes.push(pyclass)
                },
            }
        }

        Ok(Self {
            doc,
            struct_type,
            pyclass_name,
            module,
            members,
            classes,
            bases: Vec::new(),
        })
    }
}

impl ToTokens for PyClassInfo {
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
            ::pyo3_stub_gen::type_info::PyClassInfo {
                pyclass_name: #pyclass_name,
                struct_id: std::any::TypeId::of::<#struct_type>,
                members: &[ #( #members),* ],
                module: #module,
                doc: #doc,
                classes: &[ #( #classes ),* ],
                bases: &[ #( <#bases as ::pyo3_stub_gen::PyStubType>::type_output ),* ],
            }
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use syn::parse_str;

    #[test]
    fn test_pyclass() -> Result<()> {
        let input: ItemStruct = parse_str(
            r#"
            #[pyclass(mapping, module = "my_module", name = "Placeholder")]
            #[derive(
                Debug, Clone, PyNeg, PyAdd, PySub, PyMul, PyDiv, PyMod, PyPow, PyCmp, PyIndex, PyPrint,
            )]
            pub struct PyPlaceholder {
                #[pyo3(get)]
                pub name: String,
                #[pyo3(get)]
                pub ndim: usize,
                #[pyo3(get)]
                pub description: Option<String>,
                pub custom_latex: Option<String>,
            }
            "#,
        )?;
        let out = PyClassInfo::try_from(input)?.to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::PyClassInfo {
            pyclass_name: "Placeholder",
            struct_id: std::any::TypeId::of::<PyPlaceholder>,
            members: &[
                ::pyo3_stub_gen::type_info::MemberInfo {
                    name: "name",
                    r#type: <String as ::pyo3_stub_gen::PyStubType>::type_output,
                    doc: "",
                },
                ::pyo3_stub_gen::type_info::MemberInfo {
                    name: "ndim",
                    r#type: <usize as ::pyo3_stub_gen::PyStubType>::type_output,
                    doc: "",
                },
                ::pyo3_stub_gen::type_info::MemberInfo {
                    name: "description",
                    r#type: <Option<String> as ::pyo3_stub_gen::PyStubType>::type_output,
                    doc: "",
                },
            ],
            module: Some("my_module"),
            doc: "",
            classes: &[],
            bases: &[],
        }
        "###);
        Ok(())
    }

    #[test]
    fn test_complex_enum() -> Result<()> {
        let input: ItemEnum = parse_str(
            r#"
            #[pyclass(mapping, module = "my_module", name = "Placeholder")]
            #[derive(
                Debug, Clone, PyNeg, PyAdd, PySub, PyMul, PyDiv, PyMod, PyPow, PyCmp, PyIndex, PyPrint,
            )]
            pub enum PyPlaceholder {
                name(String),
                ndim{count: usize},
                description,
            }
            "#,
        )?;
        let out = PyClassInfo::try_from(input)?.to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r#"
        ::pyo3_stub_gen::type_info::PyClassInfo {
            pyclass_name: "Placeholder",
            struct_id: std::any::TypeId::of::<PyPlaceholder>,
            members: &[
                ::pyo3_stub_gen::type_info::MemberInfo {
                    name: "name",
                    r#type: <(String) as ::pyo3_stub_gen::PyStubType>::type_output,
                    doc: "",
                },
            ],
            module: Some("my_module"),
            doc: "",
            classes: &[
                ::pyo3_stub_gen::type_info::PyClassInfo {
                    pyclass_name: "ndim",
                    struct_id: std::any::TypeId::of::<PyPlaceholder>,
                    members: &[
                        ::pyo3_stub_gen::type_info::MemberInfo {
                            name: "count",
                            r#type: <usize as ::pyo3_stub_gen::PyStubType>::type_output,
                            doc: "",
                        },
                    ],
                    module: None,
                    doc: "",
                    classes: &[],
                    bases: &[],
                },
                ::pyo3_stub_gen::type_info::PyClassInfo {
                    pyclass_name: "description",
                    struct_id: std::any::TypeId::of::<PyPlaceholder>,
                    members: &[],
                    module: None,
                    doc: "",
                    classes: &[],
                    bases: &[],
                },
            ],
            bases: &[],
        }
        "#);
        Ok(())
    }

    fn format_as_value(tt: TokenStream2) -> String {
        let ttt = quote! { const _: () = #tt; };
        let formatted = prettyplease::unparse(&syn::parse_file(&ttt.to_string()).unwrap());
        formatted
            .trim()
            .strip_prefix("const _: () = ")
            .unwrap()
            .strip_suffix(';')
            .unwrap()
            .to_string()
    }
}
