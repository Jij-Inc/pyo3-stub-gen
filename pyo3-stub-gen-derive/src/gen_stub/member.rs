use crate::gen_stub::{
    attr::{parse_gen_stub_default, parse_gen_stub_override_type, OverrideTypeAttribute},
    extract_documents,
    util::TypeOrOverride,
};

use super::{extract_return_type, parse_pyo3_attrs, Attr};

use crate::gen_stub::arg::ArgInfo;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Attribute, Error, Expr, Field, FnArg, ImplItemConst, ImplItemFn, Result};

#[derive(Debug, Clone)]
pub struct MemberInfo {
    doc: String,
    name: String,
    r#type: TypeOrOverride,
    default: Option<Expr>,
    deprecated: Option<crate::gen_stub::attr::DeprecatedInfo>,
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
        let default = parse_gen_stub_default(attrs)?;
        let doc = extract_documents(attrs).join("\n");
        let pyo3_attrs = parse_pyo3_attrs(attrs)?;
        for attr in pyo3_attrs {
            if let Attr::Getter(name) = attr {
                let fn_name = sig.ident.to_string();
                let fn_getter_name = match fn_name.strip_prefix("get_") {
                    Some(s) => s.to_owned(),
                    None => fn_name,
                };
                return Ok(MemberInfo {
                    doc,
                    name: name.unwrap_or(fn_getter_name),
                    r#type: extract_return_type(&sig.output, attrs)?
                        .expect("Getter must return a type"),
                    default,
                    deprecated: crate::gen_stub::attr::extract_deprecated(attrs),
                });
            }
        }
        unreachable!("Not a getter: {:?}", item)
    }
    pub fn new_setter(item: ImplItemFn) -> Result<Self> {
        assert!(Self::is_setter(&item.attrs)?);
        let ImplItemFn { attrs, sig, .. } = &item;
        let default = parse_gen_stub_default(attrs)?;
        let doc = extract_documents(attrs).join("\n");
        let pyo3_attrs = parse_pyo3_attrs(attrs)?;
        for attr in pyo3_attrs {
            if let Attr::Setter(name) = attr {
                let fn_name = sig.ident.to_string();
                let fn_setter_name = match fn_name.strip_prefix("set_") {
                    Some(s) => s.to_owned(),
                    None => fn_name,
                };
                let r#type = sig
                    .inputs
                    .get(1)
                    .ok_or(syn::Error::new_spanned(&item, "Setter must input a type"))
                    .and_then(|arg| {
                        if let FnArg::Typed(t) = arg {
                            Ok(match parse_gen_stub_override_type(&t.attrs)? {
                                Some(OverrideTypeAttribute { type_repr, imports }) => {
                                    TypeOrOverride::OverrideType {
                                        r#type: *t.ty.clone(),
                                        type_repr,
                                        imports,
                                    }
                                }
                                _ => TypeOrOverride::RustType {
                                    r#type: *t.ty.clone(),
                                },
                            })
                        } else {
                            Err(syn::Error::new_spanned(&item, "Setter must input a type"))
                        }
                    })?;
                return Ok(MemberInfo {
                    doc,
                    name: name.unwrap_or(fn_setter_name),
                    r#type,
                    default,
                    deprecated: crate::gen_stub::attr::extract_deprecated(attrs),
                });
            }
        }
        unreachable!("Not a setter: {:?}", item)
    }
    pub fn new_classattr_fn(item: ImplItemFn) -> Result<Self> {
        assert!(Self::is_classattr(&item.attrs)?);
        let ImplItemFn { attrs, sig, .. } = &item;
        let default = parse_gen_stub_default(attrs)?;
        let doc = extract_documents(attrs).join("\n");
        Ok(MemberInfo {
            doc,
            name: sig.ident.to_string(),
            r#type: extract_return_type(&sig.output, attrs)?.expect("Getter must return a type"),
            default,
            deprecated: crate::gen_stub::attr::extract_deprecated(attrs),
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
            r#type: TypeOrOverride::RustType { r#type: ty },
            default: Some(expr),
            deprecated: crate::gen_stub::attr::extract_deprecated(&attrs),
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
        let default = parse_gen_stub_default(&attrs)?;
        let deprecated = crate::gen_stub::attr::extract_deprecated(&attrs);
        Ok(Self {
            name: field_name.unwrap_or(ident.unwrap().to_string()),
            r#type: TypeOrOverride::RustType { r#type: ty },
            doc,
            default,
            deprecated,
        })
    }
}

impl ToTokens for MemberInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            name,
            r#type,
            doc,
            default,
            deprecated,
        } = self;
        let default = default
            .as_ref()
            .map(|value| {
                if value.to_token_stream().to_string() == "None" {
                    quote! {
                        "None".to_string()
                    }
                } else {
                    let (TypeOrOverride::RustType { r#type: ty }
                    | TypeOrOverride::OverrideType { r#type: ty, .. }) = r#type;
                    quote! {
                    let v: #ty = #value;
                    ::pyo3_stub_gen::util::fmt_py_obj(v)
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
        let deprecated_info = deprecated
            .as_ref()
            .map(|deprecated| {
                quote! {
                    Some(::pyo3_stub_gen::type_info::DeprecatedInfo {
                        since: #deprecated.since,
                        note: #deprecated.note,
                    })
                }
            })
            .unwrap_or_else(|| quote! { None });
        match r#type {
            TypeOrOverride::RustType { r#type: ty } => tokens.append_all(quote! {
                ::pyo3_stub_gen::type_info::MemberInfo {
                    name: #name,
                    r#type: <#ty as ::pyo3_stub_gen::PyStubType>::type_output,
                    doc: #doc,
                    default: #default,
                    deprecated: #deprecated_info,
                }
            }),
            TypeOrOverride::OverrideType {
                type_repr, imports, ..
            } => {
                let imports = imports.iter().collect::<Vec<&String>>();
                tokens.append_all(quote! {
                    ::pyo3_stub_gen::type_info::MemberInfo {
                        name: #name,
                        r#type: || ::pyo3_stub_gen::TypeInfo { name: #type_repr.to_string(), import: ::std::collections::HashSet::from([#(#imports.into(),)*]) },
                        doc: #doc,
                        default: #default,
                        deprecated: #deprecated_info,
                    }
                })
            }
        }
    }
}

impl From<MemberInfo> for ArgInfo {
    fn from(value: MemberInfo) -> Self {
        let MemberInfo { name, r#type, .. } = value;

        Self { name, r#type }
    }
}
