use quote::ToTokens;
use syn::{
    spanned::Spanned, FnArg, GenericArgument, PatType, PathArguments, Result, Type, TypePath,
    TypeReference,
};

use crate::gen_stub::{attr::parse_gen_stub_override_type, util::TypeOrOverride};

pub fn parse_args(iter: impl IntoIterator<Item = FnArg>) -> Result<Vec<ArgInfo>> {
    let mut args = Vec::new();
    for (n, arg) in iter.into_iter().enumerate() {
        if let FnArg::Receiver(_) = arg {
            continue;
        }
        let arg = ArgInfo::try_from(arg)?;
        let (ArgInfo {
            r#type: TypeOrOverride::RustType { r#type },
            ..
        }
        | ArgInfo {
            r#type: TypeOrOverride::OverrideType { r#type, .. },
            ..
        }) = &arg;
        // Regard the first argument with `&Bound<'_, PyType>` (classmethod
        // `cls`) or `&Bound<'_, Self>` (explicit borrowed self) as a receiver.
        if let Type::Reference(TypeReference { elem, .. }) = &r#type {
            if let Type::Path(TypePath { path, .. }) = elem.as_ref() {
                let last = path.segments.last().unwrap();
                if n == 0 && last.ident == "Bound" {
                    if let PathArguments::AngleBracketed(args, ..) = &last.arguments {
                        if let Some(GenericArgument::Type(Type::Path(TypePath {
                            path, ..
                        }))) = args.args.last()
                        {
                            let inner_last = path.segments.last().unwrap();
                            if inner_last.ident == "PyType" || inner_last.ident == "Self" {
                                continue;
                            }
                        }
                    }
                }
            }
        }
        if let Type::Path(TypePath { path, .. }) = &r#type {
            let last = path.segments.last().unwrap();
            if last.ident == "Python" {
                continue;
            }
            // Regard the first argument with `PyRef<'_, Self>` /
            // `PyRefMut<'_, Self>` / `Bound<'_, Self>` / `Py<Self>` as a
            // receiver. PyO3 accepts all four shapes for self in `#[pymethods]`.
            if n == 0
                && (last.ident == "PyRef"
                    || last.ident == "PyRefMut"
                    || last.ident == "Bound"
                    || last.ident == "Py")
            {
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
                if let Some(attr) = parse_gen_stub_override_type(&attrs)? {
                    return Ok(Self {
                        name,
                        r#type: TypeOrOverride::OverrideType {
                            r#type: (*ty).clone(),
                            type_repr: attr.type_repr,
                            imports: attr.imports,
                            rust_type_markers: vec![],
                        },
                    });
                }
                return Ok(Self {
                    name,
                    r#type: TypeOrOverride::RustType {
                        r#type: (*ty).clone(),
                    },
                });
            }

            if let syn::Pat::Wild(_) = *pat {
                return Ok(Self {
                    name: "_".to_owned(),
                    r#type: TypeOrOverride::RustType { r#type: *ty },
                });
            }
        }
        Err(syn::Error::new(span, "Expected typed argument"))
    }
}
