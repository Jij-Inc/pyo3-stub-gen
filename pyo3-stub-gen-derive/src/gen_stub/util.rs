use indexmap::IndexSet;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    Attribute, GenericArgument, PathArguments, PathSegment, Result, ReturnType, Type, TypePath,
};

use crate::gen_stub::attr::parse_gen_stub_override_return_type;

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
    attrs: &[Attribute],
) -> Result<Option<TypeOrOverride>> {
    let ret = if let ReturnType::Type(_, ty) = ret {
        unwrap_pyresult(ty)
    } else {
        return Ok(None);
    };
    let mut ret = ret.clone();
    remove_lifetime(&mut ret);
    if let Some(attr) = parse_gen_stub_override_return_type(attrs)? {
        return Ok(Some(TypeOrOverride::OverrideType {
            r#type: ret.clone(),
            type_repr: attr.type_repr,
            imports: attr.imports,
        }));
    }
    Ok(Some(TypeOrOverride::RustType { r#type: ret }))
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
        imports: IndexSet<String>,
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
