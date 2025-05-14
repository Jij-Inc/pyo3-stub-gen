use crate::gen_stub::extract_documents;

use super::{escape_return_type, parse_pyo3_attrs, Attr};

use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Attribute, Error, Expr, Field, FnArg, ImplItemConst, ImplItemFn, Result, Type};

#[derive(Debug)]
pub struct MemberInfo {
    doc: String,
    name: String,
    r#type: Type,
    default: Option<Expr>,
}

impl MemberInfo {
    pub fn is_getter(attrs: &[Attribute]) -> Result<bool> {
        let attrs = parse_pyo3_attrs(attrs)?;
        Ok(attrs.iter().any(|attr| matches!(attr, Attr::Getter(_))))
    }
    pub fn is_setter(attrs: &[Attribute]) -> Result<bool> {
        let attrs = parse_pyo3_attrs(attrs)?;
        Ok(attrs.iter().any(|attr| matches!(attr, Attr::Setter(_))))
    }

    pub fn is_classattr(attrs: &[Attribute]) -> Result<bool> {
        let attrs = parse_pyo3_attrs(attrs)?;
        Ok(attrs.iter().any(|attr| matches!(attr, Attr::ClassAttr)))
    }
    pub fn is_get(field: &Field) -> Result<bool> {
        let Field { attrs, .. } = field;
        Ok(parse_pyo3_attrs(attrs)?
            .iter()
            .any(|attr| matches!(attr, Attr::Get)))
    }
    pub fn is_set(field: &Field) -> Result<bool> {
        let Field { attrs, .. } = field;
        Ok(parse_pyo3_attrs(attrs)?
            .iter()
            .any(|attr| matches!(attr, Attr::Set)))
    }
}

impl MemberInfo {
    pub fn new_getter(item: ImplItemFn) -> Result<Self> {
        assert!(Self::is_getter(&item.attrs)?);
        let ImplItemFn { attrs, sig, .. } = &item;
        let doc = extract_documents(attrs).join("\n");
        let attrs = parse_pyo3_attrs(attrs)?;
        for attr in attrs {
            if let Attr::Getter(name) = attr {
                return Ok(MemberInfo {
                    doc,
                    name: name.unwrap_or(sig.ident.to_string()),
                    r#type: escape_return_type(&sig.output).expect("Getter must return a type"),
                    default: None,
                });
            }
        }
        unreachable!("Not a getter: {:?}", item)
    }
    pub fn new_setter(item: ImplItemFn) -> Result<Self> {
        assert!(Self::is_setter(&item.attrs)?);
        let ImplItemFn { attrs, sig, .. } = &item;
        let doc = extract_documents(attrs).join("\n");
        let attrs = parse_pyo3_attrs(attrs)?;
        for attr in attrs {
            if let Attr::Getter(name) = attr {
                return Ok(MemberInfo {
                    doc,
                    name: name.unwrap_or(sig.ident.to_string()),
                    r#type: sig
                        .inputs
                        .get(1)
                        .and_then(|arg| {
                            if let FnArg::Typed(t) = arg {
                                Some(*t.ty.clone())
                            } else {
                                None
                            }
                        })
                        .expect("Setter must input a type"),
                    default: None,
                });
            }
        }
        unreachable!("Not a setter: {:?}", item)
    }
    pub fn new_classattr_fn(item: ImplItemFn) -> Result<Self> {
        assert!(Self::is_classattr(&item.attrs)?);
        let ImplItemFn { attrs, sig, .. } = &item;
        let doc = extract_documents(attrs).join("\n");
        Ok(MemberInfo {
            doc,
            name: sig.ident.to_string(),
            r#type: escape_return_type(&sig.output).expect("Getter must return a type"),
            default: None,
        })
    }
    pub fn new_classattr_const(item: ImplItemConst) -> Result<Self> {
        assert!(Self::is_classattr(&item.attrs)?);
        let ImplItemConst {
            attrs,
            ident,
            ty,
            expr,
            ..
        } = item;
        let doc = extract_documents(&attrs).join("\n");
        Ok(MemberInfo {
            doc,
            name: ident.to_string(),
            r#type: ty,
            default: Some(expr),
        })
    }
}

impl TryFrom<Field> for MemberInfo {
    type Error = Error;
    fn try_from(field: Field) -> Result<Self> {
        let Field {
            ident, ty, attrs, ..
        } = field;
        let mut field_name = None;
        for attr in parse_pyo3_attrs(&attrs)? {
            if let Attr::Name(name) = attr {
                field_name = Some(name);
            }
        }
        let doc = extract_documents(&attrs).join("\n");
        Ok(Self {
            name: field_name.unwrap_or(ident.unwrap().to_string()),
            r#type: ty,
            doc,
            default: None,
        })
    }
}

impl ToTokens for MemberInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            name,
            r#type: ty,
            doc,
            default,
        } = self;
        let name = name.strip_prefix("get_").unwrap_or(name);
        let default = default
            .as_ref()
            .map(|value| {
                if value.to_token_stream().to_string() == "None" {
                    quote! {
                        "None".to_string()
                    }
                } else {
                    quote! {
                        ::pyo3::prepare_freethreaded_python();
                        ::pyo3::Python::with_gil(|py| -> String {
                            let v: #ty = #value;
                            ::pyo3_stub_gen::util::fmt_py_obj(py, v)
                        })
                    }
                }
            })
            .map_or(quote! {None}, |default| {
                quote! {Some({
                    static DEFAULT: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
                        #default
                    });
                    &DEFAULT
                })}
            });
        tokens.append_all(quote! {
            ::pyo3_stub_gen::type_info::MemberInfo {
                name: #name,
                r#type: <#ty as ::pyo3_stub_gen::PyStubType>::type_output,
                doc: #doc,
                default: #default,
            }
        })
    }
}
