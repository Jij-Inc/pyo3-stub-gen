use super::remove_lifetime;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    spanned::Spanned, FnArg, GenericArgument, PatType, PathArguments, Result, Type, TypePath,
};

pub fn parse_args(iter: impl IntoIterator<Item = FnArg>) -> Result<Vec<ArgInfo>> {
    let mut args = Vec::new();
    for (n, arg) in iter.into_iter().enumerate() {
        if let FnArg::Receiver(_) = arg {
            continue;
        }
        let arg = ArgInfo::try_from(arg)?;
        if let Type::Path(TypePath { path, .. }) = &arg.r#type {
            let last = path.segments.last().unwrap();
            if last.ident == "Python" {
                continue;
            }
            // Regard the first argument with `PyRef<'_, Self>` and `PyMutRef<'_, Self>` types as a receiver.
            if n == 0 && (last.ident == "PyRef" || last.ident == "PyRefMut") {
                if let PathArguments::AngleBracketed(inner) = &last.arguments {
                    if let GenericArgument::Type(Type::Path(TypePath { path, .. })) =
                        &inner.args[inner.args.len() - 1]
                    {
                        let last = path.segments.last().unwrap();
                        if last.ident == "Self" {
                            continue;
                        }
                    }
                }
            }
        }
        args.push(arg);
    }
    Ok(args)
}

#[derive(Debug)]
pub struct ArgInfo {
    pub name: String,
    pub r#type: Type,
}

impl TryFrom<FnArg> for ArgInfo {
    type Error = syn::Error;
    fn try_from(value: FnArg) -> Result<Self> {
        let span = value.span();
        if let FnArg::Typed(PatType { pat, ty, .. }) = value {
            if let syn::Pat::Ident(mut ident) = *pat {
                ident.mutability = None;
                let name = ident.to_token_stream().to_string();
                return Ok(Self { name, r#type: *ty });
            }
        }
        Err(syn::Error::new(span, "Expected typed argument"))
    }
}

impl ToTokens for ArgInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self { name, r#type: ty } = self;
        let mut ty = ty.clone();
        remove_lifetime(&mut ty);
        tokens.append_all(quote! {
            ::pyo3_stub_gen::type_info::ArgInfo {
                name: #name,
                r#type: <#ty as ::pyo3_stub_gen::PyStubType>::type_input
            }
        });
    }
}
