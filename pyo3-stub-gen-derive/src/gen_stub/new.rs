use super::{
    check_specified_signature, parse_args, parse_pyo3_attrs, quote_option, ArgInfo, Attr, Signature,
};

use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Error, ImplItemFn, Result};

#[derive(Debug)]
pub struct NewInfo {
    args: Vec<ArgInfo>,
    sig: Option<Signature>,
    specified_sig: Option<String>,
}

impl NewInfo {
    pub fn is_candidate(item: &ImplItemFn) -> Result<bool> {
        let attrs = parse_pyo3_attrs(&item.attrs)?;
        Ok(attrs.iter().any(|attr| matches!(attr, Attr::New)))
    }
}

impl TryFrom<ImplItemFn> for NewInfo {
    type Error = Error;
    fn try_from(item: ImplItemFn) -> Result<Self> {
        assert!(Self::is_candidate(&item)?);
        let ImplItemFn { attrs, sig, .. } = item;
        let attrs = parse_pyo3_attrs(&attrs)?;
        let mut new_sig = None;
        let mut new_specified_sig = None;
        for attr in attrs {
            match attr {
                Attr::Signature(text_sig) => new_sig = Some(text_sig),
                Attr::SpecifiedSignature(specified_sig) => new_specified_sig = Some(specified_sig),
                _ => {}
            }
        }
        let args = parse_args(sig.inputs)?;
        check_specified_signature("__new__", &new_specified_sig, &args, &new_sig)?;
        Ok(NewInfo {
            args,
            sig: new_sig,
            specified_sig: new_specified_sig,
        })
    }
}

impl ToTokens for NewInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            args,
            sig,
            specified_sig,
        } = self;
        let sig_tt = quote_option(sig);
        let specified_sig_tt = quote_option(specified_sig);
        tokens.append_all(quote! {
            ::pyo3_stub_gen::type_info::NewInfo {
                args: &[ #(#args),* ],
                signature: #sig_tt,
                specified_signature: #specified_sig_tt,
            }
        })
    }
}
