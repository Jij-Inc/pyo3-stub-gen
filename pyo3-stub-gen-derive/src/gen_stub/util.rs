use std::collections::HashSet;

use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Attribute, GenericArgument, LitStr, PathArguments, PathSegment, Result, ReturnType, Token,
    Type, TypePath,
};

pub fn quote_option<T: ToTokens>(a: &Option<T>) -> TokenStream2 {
    if let Some(a) = a {
        quote! { Some(#a) }
    } else {
        quote! { None }
    }
}

pub fn remove_lifetime(ty: &mut Type) {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            if let Some(PathSegment {
                arguments: PathArguments::AngleBracketed(inner),
                ..
            }) = path.segments.last_mut()
            {
                for arg in &mut inner.args {
                    match arg {
                        GenericArgument::Lifetime(l) => {
                            // `T::<'a, S>` becomes `T::<'_, S>`
                            *l = syn::parse_quote!('_);
                        }
                        GenericArgument::Type(ty) => {
                            remove_lifetime(ty);
                        }
                        _ => {}
                    }
                }
            }
        }
        Type::Reference(rty) => {
            rty.lifetime = None;
            remove_lifetime(rty.elem.as_mut());
        }
        Type::Tuple(ty) => {
            for elem in &mut ty.elems {
                remove_lifetime(elem);
            }
        }
        Type::Array(ary) => {
            remove_lifetime(ary.elem.as_mut());
        }
        _ => {}
    }
}

/// Extract `T` from `PyResult<T>` and apply `override_type` attribute if present.
///
/// For `PyResult<&'a T>` case, `'a` will be removed, i.e. returns `&T` for this case.
pub fn extract_return_type(
    ret: &ReturnType,
    attrs: &Vec<Attribute>,
) -> Result<Option<TypeOrOverride>> {
    let ret = if let ReturnType::Type(_, ty) = ret {
        unwrap_pyresult(ty)
    } else {
        return Ok(None);
    };
    let mut ret = ret.clone();
    remove_lifetime(&mut ret);
    if let Some(r#type) = parse_override_type_attribute(ret.clone(), &attrs)? {
        return Ok(Some(r#type));
    }
    Ok(Some(TypeOrOverride::RustType { r#type: ret }))
}

/// Parse `override_type` attribute if present.
pub fn parse_override_type_attribute(
    r#type: Type,
    attrs: &Vec<Attribute>,
) -> Result<Option<TypeOrOverride>> {
    for attr in attrs {
        if attr.path().is_ident("override_type") {
            let attr: OverrideTypeAttribute = attr.parse_args()?;
            return Ok(Some(TypeOrOverride::OverrideType {
                r#type,
                type_repr: attr.type_repr,
                imports: attr.imports,
            }));
        }
    }
    Ok(None)
}

pub struct OverrideTypeAttribute {
    pub(crate) type_repr: String,
    pub(crate) imports: HashSet<String>,
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

fn unwrap_pyresult(ty: &Type) -> &Type {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(last) = path.segments.last() {
            if last.ident == "PyResult" {
                if let PathArguments::AngleBracketed(inner) = &last.arguments {
                    for arg in &inner.args {
                        if let GenericArgument::Type(ty) = arg {
                            return ty;
                        }
                    }
                }
            }
        }
    }
    ty
}

#[derive(Debug, Clone)]
pub enum TypeOrOverride {
    RustType {
        r#type: Type,
    },
    OverrideType {
        r#type: Type,
        type_repr: String,
        imports: HashSet<String>,
    },
}

#[cfg(test)]
mod test {
    use super::*;
    use syn::{parse_str, Result};

    #[test]
    fn test_unwrap_pyresult() -> Result<()> {
        let ty: Type = parse_str("PyResult<i32>")?;
        let out = unwrap_pyresult(&ty);
        assert_eq!(out, &parse_str("i32")?);

        let ty: Type = parse_str("PyResult<&PyString>")?;
        let out = unwrap_pyresult(&ty);
        assert_eq!(out, &parse_str("&PyString")?);

        let ty: Type = parse_str("PyResult<&'a PyString>")?;
        let out = unwrap_pyresult(&ty);
        assert_eq!(out, &parse_str("&'a PyString")?);

        let ty: Type = parse_str("::pyo3::PyResult<i32>")?;
        let out = unwrap_pyresult(&ty);
        assert_eq!(out, &parse_str("i32")?);

        let ty: Type = parse_str("::pyo3::PyResult<&PyString>")?;
        let out = unwrap_pyresult(&ty);
        assert_eq!(out, &parse_str("&PyString")?);

        let ty: Type = parse_str("::pyo3::PyResult<&'a PyString>")?;
        let out = unwrap_pyresult(&ty);
        assert_eq!(out, &parse_str("&'a PyString")?);

        Ok(())
    }
}
