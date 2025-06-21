use quote::ToTokens;
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
        // Regard the first argument with `&Bound<'_, PyType>`
        if let Type::Reference(TypeReference { elem, .. }) = &arg.r#type {
            if let Type::Path(TypePath { path, .. }) = elem.as_ref() {
                let last = path.segments.last().unwrap();
                if n == 0 && last.ident == "Bound" {
                    if let PathArguments::AngleBracketed(args, ..) = &last.arguments {
                        if let Some(last_type) = args.args.last() {
                            if last_type.to_token_stream().to_string() == "PyType" {
                                continue;
                            }
                        }
                    }
                }
            }
        }
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
            if let syn::Pat::Wild(_) = *pat {
                return Ok(Self {
                    name: "_".to_owned(),
                    r#type: *ty,
                });
            }
        }
        Err(syn::Error::new(span, "Expected typed argument"))
    }
}
