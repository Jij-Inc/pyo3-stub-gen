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
use pyfunction::*;
use pymethods::*;
use renaming::*;
use signature::*;
use stub_type::*;
use util::*;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse2, ItemEnum, ItemFn, ItemImpl, ItemStruct, LitStr, Result};

pub fn pyclass(item: TokenStream2) -> Result<TokenStream2> {
    let mut item_struct = parse2::<ItemStruct>(item)?;
    let inner = PyClassInfo::try_from(item_struct.clone())?;
    let derive_stub_type = StubType::from(&inner);
    pyclass::prune_attrs(&mut item_struct);
    Ok(quote! {
        #item_struct
        #derive_stub_type
        pyo3_stub_gen::inventory::submit! {
            #inner
        }
    })
}

pub fn pyclass_enum(item: TokenStream2) -> Result<TokenStream2> {
    let inner = PyEnumInfo::try_from(parse2::<ItemEnum>(item.clone())?)?;
    let derive_stub_type = StubType::from(&inner);
    Ok(quote! {
        #item
        #derive_stub_type
        pyo3_stub_gen::inventory::submit! {
            #inner
        }
    })
}

pub fn pyclass_complex_enum(item: TokenStream2) -> Result<TokenStream2> {
    let inner = PyComplexEnumInfo::try_from(parse2::<ItemEnum>(item.clone())?)?;
    let derive_stub_type = StubType::from(&inner);
    Ok(quote! {
        #item
        #derive_stub_type
        pyo3_stub_gen::inventory::submit! {
            #inner
        }
    })
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
    let mut item_fn = parse2::<ItemFn>(item)?;
    let mut inner = PyFunctionInfo::try_from(item_fn.clone())?;
    let python_stub = inner.parse_attr(attr)?;
    pyfunction::prune_attrs(&mut item_fn);

    // If python parameter is provided, use it instead of auto-generated metadata
    if let Some(stub_str) = python_stub {
        let python_inner = parse_python::parse_python_function_stub(stub_str)?;
        Ok(quote! {
            #item_fn
            #[automatically_derived]
            pyo3_stub_gen::inventory::submit! {
                #python_inner
            }
        })
    } else {
        Ok(quote! {
            #item_fn
            #[automatically_derived]
            pyo3_stub_gen::inventory::submit! {
                #inner
            }
        })
    }
}

pub fn gen_function_from_python_impl(input: TokenStream2) -> Result<TokenStream2> {
    let stub_str: LitStr = parse2(input)?;
    let inner = parse_python::parse_python_function_stub(stub_str)?;
    Ok(quote! { #inner })
}

pub fn gen_methods_from_python_impl(input: TokenStream2) -> Result<TokenStream2> {
    let stub_str: LitStr = parse2(input)?;
    let inner = parse_python::parse_python_methods_stub(&stub_str)?;
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
