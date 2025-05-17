use proc_macro2::{TokenStream as TokenStream2};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse_quote, Error, ItemEnum, Result, Type};
use crate::gen_stub::variant::VariantInfo;
use super::{extract_documents, parse_pyo3_attrs, util::quote_option, Attr, StubType};

pub struct PyRichEnumInfo {
    pyclass_name: String,
    struct_type: Type,
    module: Option<String>,
    variants: Vec<VariantInfo>,
    doc: String,
    bases: Vec<Type>,
}

impl From<&PyRichEnumInfo> for StubType {
    fn from(info: &PyRichEnumInfo) -> Self {
        let PyRichEnumInfo {
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

impl TryFrom<ItemEnum> for PyRichEnumInfo {
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

        let struct_type = parse_quote!(#ident);
        let pyclass_name = pyclass_name.unwrap_or_else(|| ident.clone().to_string());

        let mut items = Vec::new();
        for variant in variants {
            items.push(VariantInfo::from_variant(variant, ident.clone(), &renaming_rule)?);
        }

        Ok(Self {
            doc,
            struct_type,
            pyclass_name,
            module,
            bases,
            variants: items,
        })
    }
}

impl ToTokens for PyRichEnumInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            pyclass_name,
            struct_type,
            variants,
            doc,
            module,
            bases,
            ..
        } = self;
        let module = quote_option(module);

        let mut members = Vec::new();
        let mut classes = Vec::new();

        for variant in variants {
            match variant {
                VariantInfo::Struct{pyclass} => classes.push(pyclass),
                VariantInfo::Tuple{tuple} => members.push(tuple),
            }
        }

        tokens.append_all(quote! {
            ::pyo3_stub_gen::type_info::PyClassTreeInfo {
                pyclass_name: #pyclass_name,
                struct_id: std::any::TypeId::of::<#struct_type>,
                members: &[ #( #members),* ],
                classes: &[ #( #classes ),* ],
                bases: &[ #( #bases ),* ],
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
    fn test_rich_enum() -> Result<()> {
        let input: ItemEnum = parse_str(
            r#"
            #[pyclass(mapping, module = "my_module", name = "Placeholder")]
            #[derive(
                Debug, Clone, PyNeg, PyAdd, PySub, PyMul, PyDiv, PyMod, PyPow, PyCmp, PyIndex, PyPrint,
            )]
            pub enum PyPlaceholder {
                #[pyo3(name="Name")]
                name(String),
                twonum(i32,f64),
                ndim{count: usize},
                description,
            }
            "#,
        )?;
        let out = PyRichEnumInfo::try_from(input)?.to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r#"
        ::pyo3_stub_gen::type_info::PyClassTreeInfo {
            pyclass_name: "Placeholder",
            struct_id: std::any::TypeId::of::<PyPlaceholder>,
            members: &[
                ::pyo3_stub_gen::type_info::MemberInfo {
                    name: "Name",
                    r#type: <(String,) as ::pyo3_stub_gen::PyStubType>::type_output,
                    doc: "",
                },
                ::pyo3_stub_gen::type_info::MemberInfo {
                    name: "twonum",
                    r#type: <(i32, f64) as ::pyo3_stub_gen::PyStubType>::type_output,
                    doc: "",
                },
            ],
            classes: &[
                ::pyo3_stub_gen::type_info::PyClassTreeInfo {
                    pyclass_name: "ndim",
                    struct_id: std::any::TypeId::of::<PyPlaceholder>,
                    members: &[
                        ::pyo3_stub_gen::type_info::MemberInfo {
                            name: "count",
                            r#type: <usize as ::pyo3_stub_gen::PyStubType>::type_output,
                            doc: "",
                        },
                    ],
                    classes: &[],
                    module: None,
                    doc: "",
                    bases: &[],
                },
                ::pyo3_stub_gen::type_info::PyClassTreeInfo {
                    pyclass_name: "description",
                    struct_id: std::any::TypeId::of::<PyPlaceholder>,
                    members: &[],
                    classes: &[],
                    module: None,
                    doc: "",
                    bases: &[],
                },
            ],
            bases: &[],
            module: Some("my_module"),
            doc: "",
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
