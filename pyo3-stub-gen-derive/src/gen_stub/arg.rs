use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    spanned::Spanned, FnArg, GenericArgument, PatType, PathArguments, Result, Type, TypePath,
    TypeReference,
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
                    assert!(inner.args.len() == 2);
                    if let GenericArgument::Type(Type::Path(TypePath { path, .. })) = &inner.args[1]
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
    name: String,
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

fn type_to_token(ty: &Type) -> TokenStream2 {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            if let Some(last_seg) = path.segments.last() {
                // `CompareOp` is an enum for `__richcmp__`
                // PyO3 reference: https://docs.rs/pyo3/latest/pyo3/pyclass/enum.CompareOp.html
                // PEP: https://peps.python.org/pep-0207/
                if last_seg.ident == "CompareOp" {
                    quote! { crate::stub::compare_op_type_input }
                } else {
                    quote! { <#ty as FromPyObject>::type_input }
                }
            } else {
                unreachable!("Empty path segment: {:?}", path);
            }
        }
        Type::Reference(TypeReference { elem, .. }) => {
            match elem.as_ref() {
                Type::Path(TypePath { path, .. }) => {
                    if let Some(last) = path.segments.last() {
                        match last.ident.to_string().as_str() {
                            // Types where `&T: FromPyObject` instead of `T: FromPyObject`
                            // i.e. `&str` and most of `Py*` types defined in PyO3.
                            "str" | "PyAny" | "PyString" | "PyDict" => {
                                return quote! { <#ty as FromPyObject>::type_input };
                            }
                            _ => {}
                        }
                    }
                }
                Type::Slice(_) => {
                    return quote! { <#ty as FromPyObject>::type_input };
                }
                _ => {}
            }
            type_to_token(elem)
        }
        _ => {
            quote! { <#ty as FromPyObject>::type_input }
        }
    }
}

impl ToTokens for ArgInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self { name, r#type: ty } = self;
        let type_tt = type_to_token(ty);
        tokens.append_all(quote! {
            crate::stub::ArgInfo { name: #name, r#type: #type_tt }
        });
    }
}
