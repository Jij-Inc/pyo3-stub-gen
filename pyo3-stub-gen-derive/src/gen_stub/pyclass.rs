use super::{extract_documents, parse_pyo3_attrs, util::quote_option, Attr, MemberInfo, StubType};
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse_quote, Error, ItemStruct, Result, Type};

pub struct PyClassInfo {
    pyclass_name: String,
    struct_type: Type,
    module: Option<String>,
    getters: Vec<MemberInfo>,
    setters: Vec<MemberInfo>,
    doc: String,
    bases: Vec<Type>,
    has_eq: bool,
    has_ord: bool,
    has_hash: bool,
    has_str: bool,
    subclass: bool,
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
        let mut is_set_all = false;
        let mut bases = Vec::new();
        let mut has_eq = false;
        let mut has_ord = false;
        let mut has_hash = false;
        let mut has_str = false;
        let mut subclass = false;
        for attr in parse_pyo3_attrs(&attrs)? {
            match attr {
                Attr::Name(name) => pyclass_name = Some(name),
                Attr::Module(name) => {
                    module = Some(name);
                }
                Attr::GetAll => is_get_all = true,
                Attr::SetAll => is_set_all = true,
                Attr::Extends(typ) => bases.push(typ),
                Attr::Eq => has_eq = true,
                Attr::Ord => has_ord = true,
                Attr::Hash => has_hash = true,
                Attr::Str => has_str = true,
                Attr::Subclass => subclass = true,
                _ => {}
            }
        }
        let pyclass_name = pyclass_name.unwrap_or_else(|| ident.to_string());
        let mut getters = Vec::new();
        let mut setters = Vec::new();
        for field in fields {
            if is_get_all || MemberInfo::is_get(&field)? {
                getters.push(MemberInfo::try_from(field.clone())?)
            }
            if is_set_all || MemberInfo::is_set(&field)? {
                setters.push(MemberInfo::try_from(field)?)
            }
        }
        let doc = extract_documents(&attrs).join("\n");
        Ok(Self {
            struct_type,
            pyclass_name,
            getters,
            setters,
            module,
            doc,
            bases,
            has_eq,
            has_ord,
            has_hash,
            has_str,
            subclass,
        })
    }
}

impl ToTokens for PyClassInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            pyclass_name,
            struct_type,
            getters,
            setters,
            doc,
            module,
            bases,
            has_eq,
            has_ord,
            has_hash,
            has_str,
            subclass,
        } = self;
        let module = quote_option(module);
        tokens.append_all(quote! {
            ::pyo3_stub_gen::type_info::PyClassInfo {
                pyclass_name: #pyclass_name,
                struct_id: std::any::TypeId::of::<#struct_type>,
                getters: &[ #( #getters),* ],
                setters: &[ #( #setters),* ],
                module: #module,
                doc: #doc,
                bases: &[ #( <#bases as ::pyo3_stub_gen::PyStubType>::type_output ),* ],
                has_eq: #has_eq,
                has_ord: #has_ord,
                has_hash: #has_hash,
                has_str: #has_str,
                subclass: #subclass,
            }
        })
    }
}

// `#[gen_stub(xxx)]` is not a valid proc_macro_attribute
// it's only designed to receive user's setting.
// We need to remove all `#[gen_stub(xxx)]` before print the item_struct back
pub fn prune_attrs(item_struct: &mut ItemStruct) {
    super::attr::prune_attrs(&mut item_struct.attrs);
    for field in item_struct.fields.iter_mut() {
        super::attr::prune_attrs(&mut field.attrs);
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
            getters: &[
                ::pyo3_stub_gen::type_info::MemberInfo {
                    name: "name",
                    r#type: <String as ::pyo3_stub_gen::PyStubType>::type_output,
                    doc: "",
                    default: None,
                    deprecated: None,
                },
                ::pyo3_stub_gen::type_info::MemberInfo {
                    name: "ndim",
                    r#type: <usize as ::pyo3_stub_gen::PyStubType>::type_output,
                    doc: "",
                    default: None,
                    deprecated: None,
                },
                ::pyo3_stub_gen::type_info::MemberInfo {
                    name: "description",
                    r#type: <Option<String> as ::pyo3_stub_gen::PyStubType>::type_output,
                    doc: "",
                    default: None,
                    deprecated: None,
                },
            ],
            setters: &[],
            module: Some("my_module"),
            doc: "",
            bases: &[],
            has_eq: false,
            has_ord: false,
            has_hash: false,
            has_str: false,
            subclass: false,
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
