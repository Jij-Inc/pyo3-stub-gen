use crate::gen_stub::util::TypeOrOverride;

use super::{
    arg::parse_args, attr::IgnoreTarget, extract_deprecated, extract_documents,
    extract_return_type, parse_gen_stub_type_ignore, parse_pyo3_attrs, ArgInfo, ArgsWithSignature,
    Attr, DeprecatedInfo, Signature,
};

use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    Error, GenericArgument, ImplItemFn, PathArguments, Result, Type, TypePath, TypeReference,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MethodType {
    Instance,
    Static,
    Class,
    New,
}

#[derive(Debug)]
pub struct MethodInfo {
    pub(super) name: String,
    pub(super) args: Vec<ArgInfo>,
    pub(super) sig: Option<Signature>,
    pub(super) r#return: Option<TypeOrOverride>,
    pub(super) doc: String,
    pub(super) r#type: MethodType,
    pub(super) is_async: bool,
    pub(super) deprecated: Option<DeprecatedInfo>,
    pub(super) type_ignored: Option<IgnoreTarget>,
}

fn replace_inner(ty: &mut Type, self_: &Type) {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            if let Some(last) = path.segments.last_mut() {
                if let PathArguments::AngleBracketed(arg) = &mut last.arguments {
                    for arg in &mut arg.args {
                        if let GenericArgument::Type(ty) = arg {
                            replace_inner(ty, self_);
                        }
                    }
                }
                if last.ident == "Self" {
                    *ty = self_.clone();
                }
            }
        }
        Type::Reference(TypeReference { elem, .. }) => {
            replace_inner(elem, self_);
        }
        _ => {}
    }
}

impl MethodInfo {
    pub fn replace_self(&mut self, self_: &Type) {
        for mut arg in &mut self.args {
            let (ArgInfo {
                r#type:
                    TypeOrOverride::RustType {
                        r#type: ref mut ty, ..
                    },
                ..
            }
            | ArgInfo {
                r#type:
                    TypeOrOverride::OverrideType {
                        r#type: ref mut ty, ..
                    },
                ..
            }) = &mut arg;
            replace_inner(ty, self_);
        }
        if let Some(
            TypeOrOverride::RustType { r#type: ret }
            | TypeOrOverride::OverrideType { r#type: ret, .. },
        ) = self.r#return.as_mut()
        {
            replace_inner(ret, self_);
        }
    }
}

impl TryFrom<ImplItemFn> for MethodInfo {
    type Error = Error;
    fn try_from(item: ImplItemFn) -> Result<Self> {
        let ImplItemFn { attrs, sig, .. } = item;
        let doc = extract_documents(&attrs).join("\n");
        let deprecated = extract_deprecated(&attrs);
        let type_ignored = parse_gen_stub_type_ignore(&attrs)?;
        let pyo3_attrs = parse_pyo3_attrs(&attrs)?;
        let mut method_name = None;
        let mut text_sig = Signature::overriding_operator(&sig);
        let mut method_type = MethodType::Instance;
        for attr in pyo3_attrs {
            match attr {
                Attr::Name(name) => method_name = Some(name),
                Attr::Signature(text_sig_) => text_sig = Some(text_sig_),
                Attr::StaticMethod => method_type = MethodType::Static,
                Attr::ClassMethod => method_type = MethodType::Class,
                Attr::New => method_type = MethodType::New,
                _ => {}
            }
        }
        let name = if method_type == MethodType::New {
            "__new__".to_string()
        } else {
            method_name.unwrap_or(sig.ident.to_string())
        };
        let r#return = extract_return_type(&sig.output, &attrs)?;
        Ok(MethodInfo {
            name,
            sig: text_sig,
            args: parse_args(sig.inputs)?,
            r#return,
            doc,
            r#type: method_type,
            is_async: sig.asyncness.is_some(),
            deprecated,
            type_ignored,
        })
    }
}

impl ToTokens for MethodInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            name,
            r#return: ret,
            args,
            sig,
            doc,
            r#type,
            is_async,
            deprecated,
            type_ignored,
        } = self;
        let args_with_sig = ArgsWithSignature { args, sig };
        let ret_tt = if let Some(ret) = ret {
            match ret {
                TypeOrOverride::RustType { r#type } => {
                    let ty = r#type.clone();
                    quote! { <#ty as pyo3_stub_gen::PyStubType>::type_output }
                }
                TypeOrOverride::OverrideType {
                    type_repr, imports, ..
                } => {
                    let imports = imports.iter().collect::<Vec<&String>>();
                    quote! {
                        || ::pyo3_stub_gen::TypeInfo { name: #type_repr.to_string(), import: ::std::collections::HashSet::from([#(#imports.into(),)*]) }
                    }
                }
            }
        } else {
            quote! { ::pyo3_stub_gen::type_info::no_return_type_output }
        };
        let type_tt = match r#type {
            MethodType::Instance => quote! { ::pyo3_stub_gen::type_info::MethodType::Instance },
            MethodType::Static => quote! { ::pyo3_stub_gen::type_info::MethodType::Static },
            MethodType::Class => quote! { ::pyo3_stub_gen::type_info::MethodType::Class },
            MethodType::New => quote! { ::pyo3_stub_gen::type_info::MethodType::New },
        };
        let deprecated_tt = deprecated
            .as_ref()
            .map(|d| quote! { Some(#d) })
            .unwrap_or_else(|| quote! { None });
        let type_ignored_tt = if let Some(target) = type_ignored {
            match target {
                IgnoreTarget::All => {
                    quote! { Some(::pyo3_stub_gen::type_info::IgnoreTarget::All) }
                }
                IgnoreTarget::SpecifiedLits(rules) => {
                    let rule_strs: Vec<String> = rules.iter().map(|lit| lit.value()).collect();
                    quote! {
                        Some(::pyo3_stub_gen::type_info::IgnoreTarget::Specified(
                            &[#(#rule_strs),*] as &[&str]
                        ))
                    }
                }
            }
        } else {
            quote! { None }
        };
        tokens.append_all(quote! {
            ::pyo3_stub_gen::type_info::MethodInfo {
                name: #name,
                args: #args_with_sig,
                r#return: #ret_tt,
                doc: #doc,
                r#type: #type_tt,
                is_async: #is_async,
                deprecated: #deprecated_tt,
                type_ignored: #type_ignored_tt,
            }
        })
    }
}
