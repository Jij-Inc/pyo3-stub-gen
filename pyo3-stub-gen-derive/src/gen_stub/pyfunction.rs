use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parse::{Parse, ParseStream},
    Error, ItemFn, Result, Type,
};

use super::{
    escape_return_type, extract_documents, parse_args, parse_pyo3_attrs, quote_option, ArgInfo,
    Attr, Signature,
};

pub struct PyFunctionInfo {
    name: String,
    args: Vec<ArgInfo>,
    r#return: Option<Type>,
    sig: Option<Signature>,
    doc: String,
    module: Option<String>,
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
        let args = parse_args(item.sig.inputs)?;
        let r#return = escape_return_type(&item.sig.output);
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
        } = self;
        let ret_tt = if let Some(ret) = ret {
            quote! { <#ret as pyo3_stub_gen::PyStubType>::type_output }
        } else {
            quote! { ::pyo3_stub_gen::type_info::no_return_type_output }
        };
        let sig_tt = quote_option(sig);
        let module_tt = quote_option(module);
        tokens.append_all(quote! {
            ::pyo3_stub_gen::type_info::PyFunctionInfo {
                name: #name,
                args: &[ #(#args),* ],
                r#return: #ret_tt,
                doc: #doc,
                signature: #sig_tt,
                module: #module_tt,
            }
        })
    }
}
