use quote::ToTokens;
use syn::{
    spanned::Spanned, FnArg, GenericArgument, PatType, PathArguments, Result, Type, TypePath,
};

use crate::gen_stub::util::{parse_override_type_attribute, TypeOrOverride};

pub fn parse_args(iter: impl IntoIterator<Item = FnArg>) -> Result<Vec<ArgInfo>> {
    let mut args = Vec::new();
    for (n, arg) in iter.into_iter().enumerate() {
        if let FnArg::Receiver(_) = arg {
            continue;
        }
        let arg = ArgInfo::try_from(arg)?;
        if let ArgInfo {
            r#type:
                TypeOrOverride::RustType {
                    r#type: Type::Path(TypePath { path, .. }),
                },
            ..
        }
        | ArgInfo {
            r#type:
                TypeOrOverride::OverrideType {
                    r#type: Type::Path(TypePath { path, .. }),
                    ..
                },
            ..
        } = &arg
        {
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

#[derive(Debug, Clone)]
pub struct ArgInfo {
    pub(crate) name: String,
    pub(crate) r#type: TypeOrOverride,
}

impl TryFrom<FnArg> for ArgInfo {
    type Error = syn::Error;
    fn try_from(value: FnArg) -> Result<Self> {
        let span = value.span();
        if let FnArg::Typed(PatType { pat, ty, attrs, .. }) = value {
            if let syn::Pat::Ident(mut ident) = *pat {
                ident.mutability = None;
                let name = ident.to_token_stream().to_string();
                if let Some(r#type) = parse_override_type_attribute((*ty).clone(), &attrs)? {
                    return Ok(Self { name, r#type });
                }
                Ok(Self {
                    name,
                    r#type: TypeOrOverride::RustType {
                        r#type: (*ty).clone(),
                    },
                })
            } else {
                Err(syn::Error::new(span, "Expected identifier pattern"))
            }
        } else {
            Err(syn::Error::new(span, "Expected typed argument"))
        }
    }
}
