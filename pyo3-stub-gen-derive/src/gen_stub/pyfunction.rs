use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parse::{Parse, ParseStream},
    Error, FnArg, ItemFn, Result,
};

use crate::gen_stub::util::TypeOrOverride;

use super::{
    extract_deprecated, extract_documents, extract_return_type, parse_args, parse_gen_stub_type_ignore,
    parse_pyo3_attrs, quote_option, ArgInfo, ArgsWithSignature, Attr, DeprecatedInfo, Signature,
};

pub struct PyFunctionInfo {
    name: String,
    args: Vec<ArgInfo>,
    r#return: Option<TypeOrOverride>,
    sig: Option<Signature>,
    doc: String,
    module: Option<String>,
    is_async: bool,
    deprecated: Option<DeprecatedInfo>,
    type_ignored: Option<Vec<String>>,
}

struct ModuleAttr {
    _module: syn::Ident,
    _eq_token: syn::token::Eq,
    name: syn::LitStr,
}

impl Parse for ModuleAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            _module: input.parse()?,
            _eq_token: input.parse()?,
            name: input.parse()?,
        })
    }
}

impl PyFunctionInfo {
    pub fn parse_attr(&mut self, attr: TokenStream2) -> Result<()> {
        if attr.is_empty() {
            return Ok(());
        }
        let attr: ModuleAttr = syn::parse2(attr)?;
        self.module = Some(attr.name.value());
        Ok(())
    }
}

impl TryFrom<ItemFn> for PyFunctionInfo {
    type Error = Error;
    fn try_from(item: ItemFn) -> Result<Self> {
        let doc = extract_documents(&item.attrs).join("\n");
        let deprecated = extract_deprecated(&item.attrs);
        let type_ignored = parse_gen_stub_type_ignore(&item.attrs)?;
        let args = parse_args(item.sig.inputs)?;
        let r#return = extract_return_type(&item.sig.output, &item.attrs)?;
        let mut name = None;
        let mut sig = None;
        for attr in parse_pyo3_attrs(&item.attrs)? {
            match attr {
                Attr::Name(function_name) => name = Some(function_name),
                Attr::Signature(signature) => sig = Some(signature),
                _ => {}
            }
        }
        let name = name.unwrap_or_else(|| item.sig.ident.to_string());
        Ok(Self {
            args,
            sig,
            r#return,
            name,
            doc,
            module: None,
            is_async: item.sig.asyncness.is_some(),
            deprecated,
            type_ignored,
        })
    }
}

impl ToTokens for PyFunctionInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            args,
            r#return: ret,
            name,
            doc,
            sig,
            module,
            is_async,
            deprecated,
            type_ignored,
        } = self;
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
        // let sig_tt = quote_option(sig);
        let module_tt = quote_option(module);
        let deprecated_tt = deprecated
            .as_ref()
            .map(|d| quote! { Some(#d) })
            .unwrap_or_else(|| quote! { None });
        let type_ignored_tt = if let Some(rules) = type_ignored {
            if rules.is_empty() {
                eprintln!("Warning: It is strongly recommended to explicitly specify the rules to be ignored");
            }
            let rules_vec: Vec<_> = rules.iter().map(|r| r.as_str()).collect();
            quote! { Some(&[#(#rules_vec),*] as &[&str]) }
        } else {
            quote! { None }
        };
        let args_with_sig = ArgsWithSignature { args, sig };
        tokens.append_all(quote! {
            ::pyo3_stub_gen::type_info::PyFunctionInfo {
                name: #name,
                args: #args_with_sig,
                r#return: #ret_tt,
                doc: #doc,
                module: #module_tt,
                is_async: #is_async,
                deprecated: #deprecated_tt,
                type_ignored: #type_ignored_tt,
            }
        })
    }
}

// `#[gen_stub(xxx)]` is not a valid proc_macro_attribute
// it's only designed to receive user's setting.
// We need to remove all `#[gen_stub(xxx)]` before print the item_fn back
pub fn prune_attrs(item_fn: &mut ItemFn) {
    super::attr::prune_attrs(&mut item_fn.attrs);
    for arg in item_fn.sig.inputs.iter_mut() {
        if let FnArg::Typed(ref mut pat_type) = arg {
            super::attr::prune_attrs(&mut pat_type.attrs);
        }
    }
}
