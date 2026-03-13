//! Code generation for embedding metadata for generating Python stub file.
//!
//! These metadata are embedded as `inventory::submit!` block like:
//!
//! ```rust
//! # use pyo3::*;
//! # use pyo3_stub_gen::type_info::*;
//! # struct PyPlaceholder;
//! inventory::submit!{
//!     PyClassInfo {
//!         pyclass_name: "Placeholder",
//!         module: Some("my_module"),
//!         struct_id: std::any::TypeId::of::<PyPlaceholder>,
//!         getters: &[
//!             MemberInfo {
//!                 name: "name",
//!                 r#type: <String as ::pyo3_stub_gen::PyStubType>::type_output,
//!                 doc: "",
//!                 default: None,
//!                 deprecated: None,
//!             },
//!             MemberInfo {
//!                 name: "ndim",
//!                 r#type: <usize as ::pyo3_stub_gen::PyStubType>::type_output,
//!                 doc: "",
//!                 default: None,
//!                 deprecated: None,
//!             },
//!             MemberInfo {
//!                 name: "description",
//!                 r#type: <Option<String> as ::pyo3_stub_gen::PyStubType>::type_output,
//!                 doc: "",
//!                 default: None,
//!                 deprecated: None,
//!             },
//!         ],
//!         setters: &[],
//!         doc: "",
//!         bases: &[],
//!         has_eq: false,
//!         has_ord: false,
//!         has_hash: false,
//!         has_str: false,
//!         subclass: false,
//!     }
//! }
//! ```
//!
//! and this submodule responsible for generating such codes from Rust code like
//!
//! ```rust
//! # use pyo3::*;
//! #[pyclass(mapping, module = "my_module", name = "Placeholder")]
//! #[derive(Debug, Clone)]
//! pub struct PyPlaceholder {
//!     #[pyo3(get)]
//!     pub name: String,
//!     #[pyo3(get)]
//!     pub ndim: usize,
//!     #[pyo3(get)]
//!     pub description: Option<String>,
//!     pub custom_latex: Option<String>,
//! }
//! ```
//!
//! Mechanism
//! ----------
//! Code generation will take three steps:
//!
//! 1. Parse input [proc_macro2::TokenStream] into corresponding syntax tree component in [syn],
//!    - e.g. [ItemStruct] for `#[pyclass]`, [ItemImpl] for `#[pymethods]`, and so on.
//! 2. Convert syntax tree components into `*Info` struct using [TryInto].
//!    - e.g. [PyClassInfo] is converted from [ItemStruct], [PyMethodsInfo] is converted from [ItemImpl], and so on.
//! 3. Generate token streams using implementation of [quote::ToTokens] trait for `*Info` structs.
//!    - [quote::quote!] macro uses this trait.
//!

mod arg;
mod attr;
mod member;
mod method;
mod parameter;
mod parse_python;
mod pyclass;
mod pyclass_complex_enum;
mod pyclass_enum;
mod pyfunction;
mod pymethods;
mod renaming;
mod signature;
mod stub_type;
mod util;
mod variant;

use arg::*;
use attr::*;
use member::*;
use method::*;
use pyclass::*;
use pyclass_complex_enum::*;
use pyclass_enum::*;
use pymethods::*;
use renaming::*;
use signature::*;
use stub_type::*;
use util::*;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse2, ItemEnum, ItemFn, ItemImpl, ItemStruct, LitStr, Result};

pub fn pyclass(attr: TokenStream2, item: TokenStream2) -> Result<TokenStream2> {
    let attr = parse2::<attr::PyClassAttr>(attr)?;
    let mut item_struct = parse2::<ItemStruct>(item)?;
    let inner = PyClassInfo::from_item_with_attr(item_struct.clone(), &attr)?;
    pyclass::prune_attrs(&mut item_struct);

    if attr.skip_stub_type {
        Ok(quote! {
            #item_struct
            pyo3_stub_gen::inventory::submit! {
                #inner
            }
        })
    } else {
        let derive_stub_type = StubType::from(&inner);
        Ok(quote! {
            #item_struct
            #derive_stub_type
            pyo3_stub_gen::inventory::submit! {
                #inner
            }
        })
    }
}

pub fn pyclass_enum(attr: TokenStream2, item: TokenStream2) -> Result<TokenStream2> {
    let attr = parse2::<attr::PyClassAttr>(attr)?;
    let inner = PyEnumInfo::from_item_with_attr(parse2::<ItemEnum>(item.clone())?, &attr)?;

    if attr.skip_stub_type {
        Ok(quote! {
            #item
            pyo3_stub_gen::inventory::submit! {
                #inner
            }
        })
    } else {
        let derive_stub_type = StubType::from(&inner);
        Ok(quote! {
            #item
            #derive_stub_type
            pyo3_stub_gen::inventory::submit! {
                #inner
            }
        })
    }
}

pub fn pyclass_complex_enum(attr: TokenStream2, item: TokenStream2) -> Result<TokenStream2> {
    let attr = parse2::<attr::PyClassAttr>(attr)?;
    let inner = PyComplexEnumInfo::from_item_with_attr(parse2::<ItemEnum>(item.clone())?, &attr)?;

    if attr.skip_stub_type {
        Ok(quote! {
            #item
            pyo3_stub_gen::inventory::submit! {
                #inner
            }
        })
    } else {
        let derive_stub_type = StubType::from(&inner);
        Ok(quote! {
            #item
            #derive_stub_type
            pyo3_stub_gen::inventory::submit! {
                #inner
            }
        })
    }
}

pub fn pymethods(item: TokenStream2) -> Result<TokenStream2> {
    let mut item_impl = parse2::<ItemImpl>(item)?;
    let inner = PyMethodsInfo::try_from(item_impl.clone())?;
    pymethods::prune_attrs(&mut item_impl);
    Ok(quote! {
        #item_impl
        #[automatically_derived]
        pyo3_stub_gen::inventory::submit! {
            #inner
        }
    })
}

pub fn pyfunction(attr: TokenStream2, item: TokenStream2) -> Result<TokenStream2> {
    // Step 1: Parse TokenStream to syn types
    let item_fn = parse2::<ItemFn>(item)?;
    let attr = parse2::<pyfunction::PyFunctionAttr>(attr)?;

    // Step 2: Convert to intermediate representation
    let infos = pyfunction::PyFunctionInfos::from_parts(item_fn, attr)?;

    // Step 3: Generate output TokenStream via ToTokens
    Ok(quote! { #infos })
}

pub fn gen_function_from_python_impl(input: TokenStream2) -> Result<TokenStream2> {
    let parsed: parse_python::GenFunctionFromPythonInput = parse2(input)?;
    let inner = parse_python::parse_gen_function_from_python_input(parsed)?;
    Ok(quote! { #inner })
}

pub fn gen_methods_from_python_impl(input: TokenStream2) -> Result<TokenStream2> {
    let stub_str: LitStr = parse2(input)?;
    let inner = parse_python::parse_python_methods_stub(&stub_str)?;
    Ok(quote! { #inner })
}

pub fn gen_type_alias_from_python_impl(input: TokenStream2) -> Result<TokenStream2> {
    let parsed: parse_python::GenTypeAliasFromPythonInput = parse2(input)?;
    let inner = parse_python::parse_python_type_alias_stub(&parsed)?;
    Ok(quote! { #inner })
}

pub fn prune_gen_stub(item: TokenStream2) -> Result<TokenStream2> {
    fn prune_attrs<T: syn::parse::Parse + quote::ToTokens>(
        item: &TokenStream2,
        fn_prune_attrs: fn(&mut T),
    ) -> Result<TokenStream2> {
        parse2::<T>(item.clone()).map(|mut item| {
            fn_prune_attrs(&mut item);
            quote! { #item }
        })
    }
    prune_attrs::<ItemStruct>(&item, pyclass::prune_attrs)
        .or_else(|_| prune_attrs::<ItemImpl>(&item, pymethods::prune_attrs))
        .or_else(|_| prune_attrs::<ItemFn>(&item, pyfunction::prune_attrs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    fn format_tokens(tokens: TokenStream2) -> String {
        let formatted = prettyplease::unparse(&syn::parse_file(&tokens.to_string()).unwrap());
        formatted.trim().to_string()
    }

    #[test]
    fn test_overload_example_1_expansion() {
        // Test the overload_example_1 case: python_overload + auto-generated
        // This should generate TWO PyFunctionInfo:
        // 1. From python_overload: int -> int with is_overload: true
        // 2. From Rust signature: f64 -> f64 with is_overload: true
        let attr = quote! {
            python_overload = r#"
            @overload
            def overload_example_1(x: int) -> int: ...
            "#
        };

        let item = quote! {
            #[pyfunction]
            pub fn overload_example_1(x: f64) -> f64 {
                x + 1.0
            }
        };

        let result = pyfunction(attr, item).unwrap();
        let formatted = format_tokens(result);

        insta::assert_snapshot!(formatted);
    }

    #[test]
    fn test_overload_example_2_expansion() {
        // Test the overload_example_2 case: python_overload with no_default_overload
        // This should generate TWO PyFunctionInfo (both from python_overload):
        // 1. int -> int with is_overload: true
        // 2. float -> float with is_overload: true
        // Should NOT generate overload from Rust signature (Bound<PyAny>)
        let attr = quote! {
            python_overload = r#"
            @overload
            def overload_example_2(ob: int) -> int:
                """Increments integer by 1"""

            @overload
            def overload_example_2(ob: float) -> float:
                """Increments float by 1"""
            "#,
            no_default_overload = true
        };

        let item = quote! {
            #[pyfunction]
            pub fn overload_example_2(ob: Bound<PyAny>) -> PyResult<PyObject> {
                let py = ob.py();
                Ok(ob.into_py_any(py)?)
            }
        };

        let result = pyfunction(attr, item).unwrap();
        let formatted = format_tokens(result);

        insta::assert_snapshot!(formatted);
    }

    #[test]
    fn test_regular_function_no_overload() {
        // Test a regular function without python_overload
        // This should generate ONE PyFunctionInfo with is_overload: false
        let attr = quote! {};

        let item = quote! {
            #[pyfunction]
            pub fn regular_function(x: i32) -> i32 {
                x + 1
            }
        };

        let result = pyfunction(attr, item).unwrap();
        let formatted = format_tokens(result);

        insta::assert_snapshot!(formatted);
    }
}
