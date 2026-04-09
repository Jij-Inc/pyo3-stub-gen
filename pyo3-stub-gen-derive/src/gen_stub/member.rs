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

/// Determines which `PyStubType` method to use when generating the type annotation.
#[derive(Debug, Clone, Copy)]
pub enum MemberKind {
    /// Getter or classattr: uses `type_output` (the return type)
    Getter,
    /// Setter: uses `type_input` (the parameter type)
    Setter,
    /// `#[pyo3(get, set)]` field: currently uses `type_output`.
    /// Getter and setter metadata are constructed separately with their own `MemberKind`.
    /// This variant is unused in the current implementation but kept for future use.
    #[allow(dead_code)]
    GetSet,
}

impl MemberKind {
    fn use_type_input(self) -> bool {
        matches!(self, MemberKind::Setter)
    }
}

#[derive(Debug, Clone)]
pub struct MemberInfo {
    doc: String,
    name: String,
    r#type: TypeOrOverride,
    default: Option<Expr>,
    deprecated: Option<crate::gen_stub::attr::DeprecatedInfo>,
    kind: MemberKind,
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
    /// Create a new `MemberInfo` from a getter function.
    ///
    /// The property name is determined by the following precedence:
    /// 1. `#[pyo3(name = "...")]` - explicit name via pyo3 attribute
    /// 2. `#[getter(name)]` - explicit name via getter attribute
    /// 3. Function name with `get_` prefix stripped
    ///
    /// Note: pyo3 does not allow specifying both `#[getter(name)]` and
    /// `#[pyo3(name = "...")]` at the same time (compile error: "name may only be specified once").
    pub fn new_getter(item: ImplItemFn) -> Result<Self> {
        assert!(Self::is_getter(&item.attrs)?);
        let ImplItemFn { attrs, sig, .. } = &item;
        let default = parse_gen_stub_default(attrs)?;
        let doc = extract_documents(attrs).join("\n");
        let pyo3_attrs = parse_pyo3_attrs(attrs)?;

        // First, get the name from #[getter] or #[getter(name)]
        let mut name = None;
        for attr in &pyo3_attrs {
            if let Attr::Getter(getter_name) = attr {
                let fn_name = sig.ident.to_string();
                let fn_getter_name = match fn_name.strip_prefix("get_") {
                    Some(s) => s.to_owned(),
                    None => fn_name,
                };
                name = Some(getter_name.clone().unwrap_or(fn_getter_name));
                break;
            }
        }

        // Then, check for #[pyo3(name = "...")] which takes precedence
        for attr in &pyo3_attrs {
            if let Attr::Name(pyo3_name) = attr {
                name = Some(pyo3_name.clone());
                break;
            }
        }

        let name = name.ok_or_else(|| Error::new_spanned(&item, "Not a getter"))?;
        let r#type = extract_return_type(&sig.output, attrs)?
            .ok_or_else(|| Error::new_spanned(&item, "Getter must return a type"))?;
        Ok(MemberInfo {
            doc,
            name,
            r#type,
            default,
            deprecated: crate::gen_stub::attr::extract_deprecated(attrs),
            kind: MemberKind::Getter,
        })
    }
    /// Create a new `MemberInfo` from a setter function.
    ///
    /// The property name is determined by the following precedence:
    /// 1. `#[pyo3(name = "...")]` - explicit name via pyo3 attribute
    /// 2. `#[setter(name)]` - explicit name via setter attribute
    /// 3. Function name with `set_` prefix stripped
    ///
    /// Note: pyo3 does not allow specifying both `#[setter(name)]` and
    /// `#[pyo3(name = "...")]` at the same time (compile error: "name may only be specified once").
    pub fn new_setter(item: ImplItemFn) -> Result<Self> {
        assert!(Self::is_setter(&item.attrs)?);
        let ImplItemFn { attrs, sig, .. } = &item;
        let default = parse_gen_stub_default(attrs)?;
        let doc = extract_documents(attrs).join("\n");
        let pyo3_attrs = parse_pyo3_attrs(attrs)?;

        // First, get the name from #[setter] or #[setter(name)]
        let mut name = None;
        let mut r#type = None;
        for attr in &pyo3_attrs {
            if let Attr::Setter(setter_name) = attr {
                let fn_name = sig.ident.to_string();
                let fn_setter_name = match fn_name.strip_prefix("set_") {
                    Some(s) => s.to_owned(),
                    None => fn_name,
                };
                name = Some(setter_name.clone().unwrap_or(fn_setter_name));
                r#type = Some(
                    sig.inputs
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
                                            rust_type_markers: vec![],
                                        }
                                    }
                                    _ => TypeOrOverride::RustType {
                                        r#type: *t.ty.clone(),
                                    },
                                })
                            } else {
                                Err(syn::Error::new_spanned(&item, "Setter must input a type"))
                            }
                        })?,
                );
                break;
            }
        }

        // Then, check for #[pyo3(name = "...")] which takes precedence
        for attr in &pyo3_attrs {
            if let Attr::Name(pyo3_name) = attr {
                name = Some(pyo3_name.clone());
                break;
            }
        }

        let name = name.ok_or_else(|| Error::new_spanned(&item, "Not a setter"))?;
        let r#type = r#type.ok_or_else(|| Error::new_spanned(&item, "Setter type not found"))?;
        Ok(MemberInfo {
            doc,
            name,
            r#type,
            default,
            deprecated: crate::gen_stub::attr::extract_deprecated(attrs),
            kind: MemberKind::Setter,
        })
    }
    pub fn new_classattr_fn(item: ImplItemFn) -> Result<Self> {
        assert!(Self::is_classattr(&item.attrs)?);
        let ImplItemFn { attrs, sig, .. } = &item;
        let default = parse_gen_stub_default(attrs)?;
        let doc = extract_documents(attrs).join("\n");
        let mut name = sig.ident.to_string();
        for attr in parse_pyo3_attrs(attrs)? {
            if let Attr::Name(_name) = attr {
                name = _name;
            }
        }
        Ok(MemberInfo {
            doc,
            name,
            r#type: extract_return_type(&sig.output, attrs)?.expect("Getter must return a type"),
            default,
            deprecated: crate::gen_stub::attr::extract_deprecated(attrs),
            kind: MemberKind::Getter,
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
        let mut name = ident.to_string();
        for attr in parse_pyo3_attrs(&attrs)? {
            if let Attr::Name(_name) = attr {
                name = _name;
            }
        }
        Ok(MemberInfo {
            doc,
            name,
            r#type: TypeOrOverride::RustType { r#type: ty },
            default: Some(expr),
            deprecated: crate::gen_stub::attr::extract_deprecated(&attrs),
            kind: MemberKind::Getter,
        })
    }
}

impl MemberInfo {
    pub fn from_field(field: Field, kind: MemberKind) -> Result<Self> {
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
            kind,
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
            kind,
        } = self;
        let use_type_input = kind.use_type_input();
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
                    fn _fmt() -> String {
                        #default
                    }
                    _fmt
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
        let type_fn = if use_type_input {
            quote! { type_input }
        } else {
            quote! { type_output }
        };
        match r#type {
            TypeOrOverride::RustType { r#type: ty } => tokens.append_all(quote! {
                ::pyo3_stub_gen::type_info::MemberInfo {
                    name: #name,
                    r#type: <#ty as ::pyo3_stub_gen::PyStubType>::#type_fn,
                    doc: #doc,
                    default: #default,
                    deprecated: #deprecated_info,
                }
            }),
            TypeOrOverride::OverrideType {
                type_repr,
                imports,
                rust_type_markers,
                ..
            } => {
                let imports = imports.iter().collect::<Vec<&String>>();

                // Generate code to process RustType markers
                let (type_name_code, type_refs_code) = if rust_type_markers.is_empty() {
                    (
                        quote! { #type_repr.to_string() },
                        quote! { ::std::collections::HashMap::new() },
                    )
                } else {
                    // Parse rust_type_markers as syn::Type
                    let marker_types: Vec<syn::Type> = rust_type_markers
                        .iter()
                        .filter_map(|s| syn::parse_str(s).ok())
                        .collect();

                    let rust_names = rust_type_markers.iter().collect::<Vec<_>>();

                    (
                        quote! {
                            {
                                let mut type_name = #type_repr.to_string();
                                #(
                                    let type_info = <#marker_types as ::pyo3_stub_gen::PyStubType>::type_input();
                                    type_name = type_name.replace(#rust_names, &type_info.name);
                                )*
                                type_name
                            }
                        },
                        quote! {
                            {
                                let mut type_refs = ::std::collections::HashMap::new();
                                #(
                                    let type_info = <#marker_types as ::pyo3_stub_gen::PyStubType>::type_input();
                                    if let Some(module) = type_info.source_module {
                                        type_refs.insert(
                                            type_info.name.split('[').next().unwrap_or(&type_info.name).split('.').last().unwrap_or(&type_info.name).to_string(),
                                            ::pyo3_stub_gen::TypeIdentifierRef {
                                                module: module.into(),
                                                import_kind: ::pyo3_stub_gen::ImportKind::Module,
                                            }
                                        );
                                    }
                                    type_refs.extend(type_info.type_refs);
                                )*
                                type_refs
                            }
                        },
                    )
                };

                tokens.append_all(quote! {
                    ::pyo3_stub_gen::type_info::MemberInfo {
                        name: #name,
                        r#type: || ::pyo3_stub_gen::TypeInfo { name: #type_name_code, source_module: None, import: ::std::collections::HashSet::from([#(#imports.into(),)*]), type_refs: #type_refs_code },
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
