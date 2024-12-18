use super::{
    arg::parse_args, escape_return_type, extract_documents, parse_pyo3_attrs, quote_option,
    ArgInfo, Attr, Signature,
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
    name: String,
    args: Vec<ArgInfo>,
    sig: Option<Signature>,
    r#return: Option<Type>,
    doc: String,
    r#type: MethodType,
}

fn replace_inner(ty: &mut Type, self_: &Type) {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            if let Some(last) = path.segments.iter_mut().last() {
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
        for arg in &mut self.args {
            replace_inner(&mut arg.r#type, self_);
        }
        if let Some(ret) = self.r#return.as_mut() {
            replace_inner(ret, self_);
        }
    }
}

impl TryFrom<ImplItemFn> for MethodInfo {
    type Error = Error;
    fn try_from(item: ImplItemFn) -> Result<Self> {
        let ImplItemFn { attrs, sig, .. } = item;
        let doc = extract_documents(&attrs).join("\n");
        let attrs = parse_pyo3_attrs(&attrs)?;
        let mut method_name = None;
        let mut text_sig = Signature::overriding_operator(&sig);
        let mut method_type = MethodType::Instance;
        for attr in attrs {
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
        let r#return = escape_return_type(&sig.output);
        Ok(MethodInfo {
            name,
            sig: text_sig,
            args: parse_args(sig.inputs)?,
            r#return,
            doc,
            r#type: method_type,
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
        } = self;
        let sig_tt = quote_option(sig);
        let ret_tt = if let Some(ret) = ret {
            quote! { <#ret as pyo3_stub_gen::PyStubType>::type_output }
        } else {
            quote! { ::pyo3_stub_gen::type_info::no_return_type_output }
        };
        let type_tt = match r#type {
            MethodType::Instance => quote! { ::pyo3_stub_gen::type_info::MethodType::Instance },
            MethodType::Static => quote! { ::pyo3_stub_gen::type_info::MethodType::Static },
            MethodType::Class => quote! { ::pyo3_stub_gen::type_info::MethodType::Class },
            MethodType::New => quote! { ::pyo3_stub_gen::type_info::MethodType::New },
        };
        tokens.append_all(quote! {
            ::pyo3_stub_gen::type_info::MethodInfo {
                name: #name,
                args: &[ #(#args),* ],
                r#return: #ret_tt,
                signature: #sig_tt,
                doc: #doc,
                r#type: #type_tt
            }
        })
    }
}
