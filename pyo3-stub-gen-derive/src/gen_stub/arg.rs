use std::collections::HashSet;

use quote::ToTokens;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    FnArg, GenericArgument, LitStr, PatType, PathArguments, Result, Token, Type, TypePath,
};

pub fn parse_args(iter: impl IntoIterator<Item = FnArg>) -> Result<Vec<ArgInfo>> {
    let mut args = Vec::new();
    for (n, arg) in iter.into_iter().enumerate() {
        if let FnArg::Receiver(_) = arg {
            continue;
        }
        let arg = ArgInfo::try_from(arg)?;
        if let ArgInfo::RustType {
            r#type: Type::Path(TypePath { path, .. }),
            ..
        }
        | ArgInfo::OverrideType {
            r#type: Type::Path(TypePath { path, .. }),
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
pub enum ArgInfo {
    RustType {
        name: String,
        r#type: Type,
    },
    OverrideType {
        name: String,
        r#type: Type,
        type_repr: String,
        imports: HashSet<String>,
    },
}

impl TryFrom<FnArg> for ArgInfo {
    type Error = syn::Error;
    fn try_from(value: FnArg) -> Result<Self> {
        let span = value.span();
        if let FnArg::Typed(PatType { pat, ty, attrs, .. }) = value {
            if let syn::Pat::Ident(mut ident) = *pat {
                ident.mutability = None;
                let name = ident.to_token_stream().to_string();
                for attr in &attrs {
                    if attr.path().is_ident("override_type") {
                        let attr: OverrideTypeAttribute = attr.parse_args()?;
                        return Ok(Self::OverrideType {
                            name: name,
                            r#type: *ty,
                            type_repr: attr.type_repr,
                            imports: attr.imports,
                        });
                    }
                }
                Ok(Self::RustType {
                    name,
                    r#type: (*ty).clone(),
                })
            } else {
                Err(syn::Error::new(span, "Expected identifier pattern"))
            }
        } else {
            Err(syn::Error::new(span, "Expected typed argument"))
        }
    }
}

pub struct OverrideTypeAttribute {
    type_repr: String,
    imports: HashSet<String>,
}

mod kw {
    syn::custom_keyword!(type_repr);
    syn::custom_keyword!(imports);
    syn::custom_keyword!(override_type);
}

impl Parse for OverrideTypeAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut type_repr = None;
        let mut imports = HashSet::new();

        while !input.is_empty() {
            let lookahead = input.lookahead1();

            if lookahead.peek(kw::type_repr) {
                input.parse::<kw::type_repr>()?;
                input.parse::<Token![=]>()?;
                type_repr = Some(input.parse::<LitStr>()?);
            } else if lookahead.peek(kw::imports) {
                input.parse::<kw::imports>()?;
                input.parse::<Token![=]>()?;

                let content;
                parenthesized!(content in input);
                let parsed_imports = Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?;
                imports = parsed_imports.into_iter().collect();
            } else {
                return Err(lookahead.error());
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(OverrideTypeAttribute {
            type_repr: type_repr
                .ok_or_else(|| input.error("missing type_repr"))?
                .value(),
            imports: imports.iter().map(|i| i.value()).collect(),
        })
    }
}
