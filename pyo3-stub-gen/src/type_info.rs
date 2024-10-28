//! Store of metadata for generating Python stub file
//!
//! Stub file generation takes two steps:
//!
//! Store metadata (compile time)
//! ------------------------------
//! Embed compile-time information about Rust types and PyO3 macro arguments
//! using [inventory::submit!](https://docs.rs/inventory/latest/inventory/macro.submit.html) macro into source codes,
//! and these information will be gathered by [inventory::iter](https://docs.rs/inventory/latest/inventory/struct.iter.html).
//! This submodule is responsible for this process.
//!
//! - [PyClassInfo] stores information obtained from `#[pyclass]` macro
//! - [PyMethodsInfo] stores information obtained from `#[pymethods]` macro
//!
//! and others are their components.
//!
//! Gathering metadata and generating stub file (runtime)
//! -------------------------------------------------------
//! Since `#[pyclass]` and `#[pymethods]` definitions are not bundled in a single block,
//! we have to reconstruct these information corresponding to a Python `class`.
//! This process is done at runtime in [gen_stub](../../gen_stub) executable.
//!

use crate::{PyStubType, TypeInfo};
use std::any::TypeId;

/// Work around for `CompareOp` for `__richcmp__` argument,
/// which does not implements `FromPyObject`
pub fn compare_op_type_input() -> TypeInfo {
    <isize as PyStubType>::type_input()
}

pub fn no_return_type_output() -> TypeInfo {
    TypeInfo::none()
}

/// Info of method argument appears in `#[pymethods]`
#[derive(Debug)]
pub struct ArgInfo {
    pub name: &'static str,
    pub r#type: fn() -> TypeInfo,
}

/// Info of usual method appears in `#[pymethod]`
#[derive(Debug)]
pub struct MethodInfo {
    pub name: &'static str,
    pub args: &'static [ArgInfo],
    pub r#return: fn() -> TypeInfo,
    pub signature: Option<&'static str>,
    pub doc: &'static str,
    pub is_static: bool,
    pub is_class: bool,
}

/// Info of getter method decorated with `#[getter]` or `#[pyo3(get, set)]` appears in `#[pyclass]`
#[derive(Debug)]
pub struct MemberInfo {
    pub name: &'static str,
    pub r#type: fn() -> TypeInfo,
    pub doc: &'static str,
}

/// Info of `#[new]`-attributed methods appears in `#[pymethods]`
#[derive(Debug)]
pub struct NewInfo {
    pub args: &'static [ArgInfo],
    pub signature: Option<&'static str>,
}

/// Info of `#[pymethod]`
#[derive(Debug)]
pub struct PyMethodsInfo {
    // The Rust struct type-id of `impl` block where `#[pymethod]` acts on
    pub struct_id: fn() -> TypeId,
    /// Method specified `#[new]` attribute
    pub new: Option<NewInfo>,
    /// Methods decorated with `#[getter]`
    pub getters: &'static [MemberInfo],
    /// Other usual methods
    pub methods: &'static [MethodInfo],
}

inventory::collect!(PyMethodsInfo);

/// Info of `#[pyclass]` with Rust struct
#[derive(Debug)]
pub struct PyClassInfo {
    // Rust struct type-id
    pub struct_id: fn() -> TypeId,
    // The name exposed to Python
    pub pyclass_name: &'static str,
    /// Module name specified by `#[pyclass(module = "foo.bar")]`
    pub module: Option<&'static str>,
    /// Docstring
    pub doc: &'static str,
    /// static members by `#[pyo3(get, set)]`
    pub members: &'static [MemberInfo],
}

inventory::collect!(PyClassInfo);

/// Info of `#[pyclass]` with Rust enum
#[derive(Debug)]
pub struct PyEnumInfo {
    // Rust struct type-id
    pub enum_id: fn() -> TypeId,
    // The name exposed to Python
    pub pyclass_name: &'static str,
    /// Module name specified by `#[pyclass(module = "foo.bar")]`
    pub module: Option<&'static str>,
    /// Docstring
    pub doc: &'static str,
    /// Variants of enum
    pub variants: &'static [&'static str],
}

inventory::collect!(PyEnumInfo);

/// Info of `#[pyfunction]`
#[derive(Debug)]
pub struct PyFunctionInfo {
    pub name: &'static str,
    pub args: &'static [ArgInfo],
    pub r#return: fn() -> TypeInfo,
    pub doc: &'static str,
    pub signature: Option<&'static str>,
    pub module: Option<&'static str>,
}

inventory::collect!(PyFunctionInfo);

#[derive(Debug)]
pub struct PyErrorInfo {
    pub name: &'static str,
    pub module: &'static str,
    pub base: fn() -> &'static str,
}

inventory::collect!(PyErrorInfo);

#[derive(Debug)]
pub struct PyVariableInfo {
    pub name: &'static str,
    pub module: &'static str,
    pub r#type: fn() -> TypeInfo,
}

inventory::collect!(PyVariableInfo);
