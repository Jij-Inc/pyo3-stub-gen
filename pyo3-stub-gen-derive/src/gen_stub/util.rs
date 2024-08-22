use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{GenericArgument, PathArguments, PathSegment, ReturnType, Type, TypePath};

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

/// Extract `T` from `PyResult<T>`.
///
/// For `PyResult<&'a T>` case, `'a` will be removed, i.e. returns `&T` for this case.
pub fn escape_return_type(ret: &ReturnType) -> Option<Type> {
    let ret = if let ReturnType::Type(_, ty) = ret {
        unwrap_pyresult(ty)
    } else {
        return None;
    };
    let mut ret = ret.clone();
    remove_lifetime(&mut ret);
    Some(ret)
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
