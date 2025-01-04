use super::{
    arg::parse_args, check_specified_signature, escape_return_type, extract_documents,
    parse_pyo3_attrs, quote_option, ArgInfo, Attr, Signature,
};

use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    Error, GenericArgument, ImplItemFn, PathArguments, Result, Type, TypePath, TypeReference,
};

#[derive(Debug)]
pub struct MethodInfo {
    name: String,
    args: Vec<ArgInfo>,
    sig: Option<Signature>,
    specified_sig: Option<String>,
    r#return: Option<Type>,
    doc: String,
    is_static: bool,
    is_class: bool,
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
        let mut text_sig: Option<Signature> = Signature::overriding_operator(&sig);
        let mut specified_sig = None;
        let mut is_static = false;
        let mut is_class = false;
        for attr in attrs {
            match attr {
                Attr::Name(name) => method_name = Some(name),
                Attr::Signature(text_sig_) => text_sig = Some(text_sig_),
                Attr::SpecifiedSignature(specified_sig_) => specified_sig = Some(specified_sig_),
                Attr::StaticMethod => is_static = true,
                Attr::ClassMethod => is_class = true,
                _ => {}
            }
        }
        let name = method_name.unwrap_or(sig.ident.to_string());
        let r#return = escape_return_type(&sig.output);
        let args = parse_args(sig.inputs)?;
        check_specified_signature(&name, &specified_sig, &args, &text_sig)?;
        Ok(MethodInfo {
            name,
            sig: text_sig,
            specified_sig,
            args,
            r#return,
            doc,
            is_static,
            is_class,
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
            specified_sig,
            doc,
            is_class,
            is_static,
        } = self;
        let sig_tt = quote_option(sig);
        let specified_sig_tt = quote_option(specified_sig);
        let ret_tt = if let Some(ret) = ret {
            quote! { <#ret as pyo3_stub_gen::PyStubType>::type_output }
        } else {
            quote! { ::pyo3_stub_gen::type_info::no_return_type_output }
        };
        let args_tt = if specified_sig.is_some() {
            // turn-off auto type inference when specified signature
            quote! { &[] }
        } else {
            quote! { &[ #(#args),* ] }
        };
        tokens.append_all(quote! {
            ::pyo3_stub_gen::type_info::MethodInfo {
                name: #name,
                args: #args_tt,
                r#return: #ret_tt,
                signature: #sig_tt,
                specified_signature: #specified_sig_tt,
                doc: #doc,
                is_static: #is_static,
                is_class: #is_class,
            }
        })
    }
}
