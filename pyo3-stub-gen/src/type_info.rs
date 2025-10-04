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

/// Represents the target of type ignore comments
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IgnoreTarget {
    /// Ignore all type checking errors `(# type: ignore)`
    All,
    /// Ignore specific type checking rules `(# type: ignore[rule1,rule2])`
    Specified(&'static [&'static str]),
}

/// Information about deprecated items
#[derive(Debug, Clone, PartialEq)]
pub struct DeprecatedInfo {
    pub since: Option<&'static str>,
    pub note: Option<&'static str>,
}

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
    pub signature: Option<SignatureArg>,
}
#[derive(Debug, Clone)]
pub enum SignatureArg {
    Ident,
    Assign { default: fn() -> String },
    Star,
    Args,
    Keywords,
}

impl PartialEq for SignatureArg {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Assign { default: l_default }, Self::Assign { default: r_default }) => {
                let l_default = l_default();
                let r_default = r_default();
                l_default.eq(&r_default)
            }
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

/// Type of a method
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MethodType {
    Instance,
    Static,
    Class,
    New,
}

/// Info of usual method appears in `#[pymethod]`
#[derive(Debug)]
pub struct MethodInfo {
    pub name: &'static str,
    pub args: &'static [ArgInfo],
    pub r#return: fn() -> TypeInfo,
    pub doc: &'static str,
    pub r#type: MethodType,
    pub is_async: bool,
    pub deprecated: Option<DeprecatedInfo>,
    pub type_ignored: Option<IgnoreTarget>,
}

/// Info of getter method decorated with `#[getter]` or `#[pyo3(get, set)]` appears in `#[pyclass]`
#[derive(Debug)]
pub struct MemberInfo {
    pub name: &'static str,
    pub r#type: fn() -> TypeInfo,
    pub doc: &'static str,
    pub default: Option<fn() -> String>,
    pub deprecated: Option<DeprecatedInfo>,
}

/// Info of `#[pymethod]`
#[derive(Debug)]
pub struct PyMethodsInfo {
    // The Rust struct type-id of `impl` block where `#[pymethod]` acts on
    pub struct_id: fn() -> TypeId,
    /// Method/Const with `#[classattr]`
    pub attrs: &'static [MemberInfo],
    /// Methods decorated with `#[getter]`
    pub getters: &'static [MemberInfo],
    /// Methods decorated with `#[getter]`
    pub setters: &'static [MemberInfo],
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
    /// static members by `#[pyo3(get)]`
    pub getters: &'static [MemberInfo],
    /// static members by `#[pyo3(set)]`
    pub setters: &'static [MemberInfo],
    /// Base classes specified by `#[pyclass(extends = Type)]`
    pub bases: &'static [fn() -> TypeInfo],
    /// Whether the class has eq attribute
    pub has_eq: bool,
    /// Whether the class has ord attribute
    pub has_ord: bool,
    /// Whether the class has hash attribute
    pub has_hash: bool,
    /// Whether the class has str attribute
    pub has_str: bool,
    /// Whether the class has subclass attribute `#[pyclass(subclass)]`
    pub subclass: bool,
}

inventory::collect!(PyClassInfo);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VariantForm {
    Unit,
    Tuple,
    Struct,
}

/// Info of a `#[pyclass]` with a single variant of a rich (structured) Rust enum
#[derive(Debug)]
pub struct VariantInfo {
    pub pyclass_name: &'static str,
    pub module: Option<&'static str>,
    pub doc: &'static str,
    pub fields: &'static [MemberInfo],
    pub form: &'static VariantForm,
    pub constr_args: &'static [ArgInfo],
}

/// Info of a `#[pyclass]` with a rich (structured) Rust enum
#[derive(Debug)]
pub struct PyComplexEnumInfo {
    // Rust struct type-id
    pub enum_id: fn() -> TypeId,
    // The name exposed to Python
    pub pyclass_name: &'static str,
    /// Module name specified by `#[pyclass(module = "foo.bar")]`
    pub module: Option<&'static str>,
    /// Docstring
    pub doc: &'static str,
    /// static members by `#[pyo3(get, set)]`
    pub variants: &'static [VariantInfo],
}

inventory::collect!(PyComplexEnumInfo);

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
    /// Variants of enum (name, doc)
    pub variants: &'static [(&'static str, &'static str)],
}

inventory::collect!(PyEnumInfo);

/// Info of `#[pyfunction]`
#[derive(Debug)]
pub struct PyFunctionInfo {
    pub name: &'static str,
    pub args: &'static [ArgInfo],
    pub r#return: fn() -> TypeInfo,
    pub doc: &'static str,
    pub module: Option<&'static str>,
    pub is_async: bool,
    pub deprecated: Option<DeprecatedInfo>,
    pub type_ignored: Option<IgnoreTarget>,
}

inventory::collect!(PyFunctionInfo);

#[derive(Debug)]
pub struct PyVariableInfo {
    pub name: &'static str,
    pub module: &'static str,
    pub r#type: fn() -> TypeInfo,
    pub default: Option<fn() -> String>,
}

inventory::collect!(PyVariableInfo);

#[derive(Debug)]
pub struct ModuleDocInfo {
    pub module: &'static str,
    pub doc: fn() -> String,
}

inventory::collect!(ModuleDocInfo);
