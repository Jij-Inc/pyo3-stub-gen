use super::{extract_documents, parse_pyo3_attrs, util::quote_option, Attr, StubType};
use crate::gen_stub::variant::VariantInfo;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse_quote, Error, ItemEnum, Result, Type};

pub struct PyComplexEnumInfo {
    pyclass_name: String,
    enum_type: Type,
    module: Option<String>,
    variants: Vec<VariantInfo>,
    doc: String,
}

impl From<&PyComplexEnumInfo> for StubType {
    fn from(info: &PyComplexEnumInfo) -> Self {
        let PyComplexEnumInfo {
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

impl TryFrom<ItemEnum> for PyComplexEnumInfo {
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
        let mut bases = Vec::new();
        for attr in parse_pyo3_attrs(&attrs)? {
            match attr {
                Attr::Name(name) => pyclass_name = Some(name),
                Attr::Module(name) => module = Some(name),
                Attr::RenameAll(name) => renaming_rule = Some(name),
                Attr::Extends(typ) => bases.push(typ),
                _ => {}
            }
        }

        let enum_type = parse_quote!(#ident);
        let pyclass_name = pyclass_name.unwrap_or_else(|| ident.clone().to_string());

        let mut items = Vec::new();
        for variant in variants {
            items.push(VariantInfo::from_variant(variant, &renaming_rule)?)
        }

        Ok(Self {
            doc,
            enum_type,
            pyclass_name,
            module,
            variants: items,
        })
    }
}

impl ToTokens for PyComplexEnumInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            pyclass_name,
            enum_type,
            variants,
            doc,
            module,
            ..
        } = self;
        let module = quote_option(module);

        tokens.append_all(quote! {
            ::pyo3_stub_gen::type_info::PyComplexEnumInfo {
                pyclass_name: #pyclass_name,
                enum_id: std::any::TypeId::of::<#enum_type>,
                variants: &[ #( #variants ),* ],
                module: #module,
                doc: #doc,
            }
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use syn::parse_str;

    #[test]
    fn test_complex_enum() -> Result<()> {
        let input: ItemEnum = parse_str(
            r#"
            #[pyclass(mapping, module = "my_module", name = "Placeholder")]
            #[derive(
                Debug, Clone, PyNeg, PyAdd, PySub, PyMul, PyDiv, PyMod, PyPow, PyCmp, PyIndex, PyPrint,
            )]
            pub enum PyPlaceholder {
                #[pyo3(name="Name")]
                name(String),
                #[pyo3(constructor = (_0, _1=1.0))]
                twonum(i32,f64),
                ndim{count: usize},
                description,
            }
            "#,
        )?;
        let out = PyComplexEnumInfo::try_from(input)?.to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::PyComplexEnumInfo {
            pyclass_name: "Placeholder",
            enum_id: std::any::TypeId::of::<PyPlaceholder>,
            variants: &[
                ::pyo3_stub_gen::type_info::VariantInfo {
                    pyclass_name: "Name",
                    fields: &[
                        ::pyo3_stub_gen::type_info::MemberInfo {
                            name: "_0",
                            r#type: <String as ::pyo3_stub_gen::PyStubType>::type_output,
                            doc: "",
                            default: None,
                            deprecated: None,
                        },
                    ],
                    module: None,
                    doc: "",
                    form: &pyo3_stub_gen::type_info::VariantForm::Tuple,
                    constr_args: &[
                        ::pyo3_stub_gen::type_info::ParameterInfo {
                            name: "_0",
                            kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                            type_info: <String as ::pyo3_stub_gen::PyStubType>::type_input,
                            default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                        },
                    ],
                },
                ::pyo3_stub_gen::type_info::VariantInfo {
                    pyclass_name: "twonum",
                    fields: &[
                        ::pyo3_stub_gen::type_info::MemberInfo {
                            name: "_0",
                            r#type: <i32 as ::pyo3_stub_gen::PyStubType>::type_output,
                            doc: "",
                            default: None,
                            deprecated: None,
                        },
                        ::pyo3_stub_gen::type_info::MemberInfo {
                            name: "_1",
                            r#type: <f64 as ::pyo3_stub_gen::PyStubType>::type_output,
                            doc: "",
                            default: None,
                            deprecated: None,
                        },
                    ],
                    module: None,
                    doc: "",
                    form: &pyo3_stub_gen::type_info::VariantForm::Tuple,
                    constr_args: &[
                        ::pyo3_stub_gen::type_info::ParameterInfo {
                            name: "_0",
                            kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                            type_info: <i32 as ::pyo3_stub_gen::PyStubType>::type_input,
                            default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                        },
                        ::pyo3_stub_gen::type_info::ParameterInfo {
                            name: "_1",
                            kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                            type_info: <f64 as ::pyo3_stub_gen::PyStubType>::type_input,
                            default: ::pyo3_stub_gen::type_info::ParameterDefault::Expr({
                                fn _fmt() -> String {
                                    let v: f64 = 1.0;
                                    ::pyo3_stub_gen::util::fmt_py_obj(v)
                                }
                                _fmt
                            }),
                        },
                    ],
                },
                ::pyo3_stub_gen::type_info::VariantInfo {
                    pyclass_name: "ndim",
                    fields: &[
                        ::pyo3_stub_gen::type_info::MemberInfo {
                            name: "count",
                            r#type: <usize as ::pyo3_stub_gen::PyStubType>::type_output,
                            doc: "",
                            default: None,
                            deprecated: None,
                        },
                    ],
                    module: None,
                    doc: "",
                    form: &pyo3_stub_gen::type_info::VariantForm::Struct,
                    constr_args: &[
                        ::pyo3_stub_gen::type_info::ParameterInfo {
                            name: "count",
                            kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                            type_info: <usize as ::pyo3_stub_gen::PyStubType>::type_input,
                            default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                        },
                    ],
                },
                ::pyo3_stub_gen::type_info::VariantInfo {
                    pyclass_name: "description",
                    fields: &[],
                    module: None,
                    doc: "",
                    form: &pyo3_stub_gen::type_info::VariantForm::Unit,
                    constr_args: &[],
                },
            ],
            module: Some("my_module"),
            doc: "",
        }
        "###);
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
