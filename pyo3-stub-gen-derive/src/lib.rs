mod gen_stub;

use proc_macro::TokenStream;

/// Embed metadata for Python stub file generation for `#[pyclass]` macro
///
/// ```
/// #[pyo3_stub_gen_derive::gen_stub_pyclass]
/// #[pyo3::pyclass(mapping, module = "my_module", name = "Placeholder")]
/// #[derive(Debug, Clone)]
/// pub struct PyPlaceholder {
///     #[pyo3(get)]
///     pub name: String,
///     #[pyo3(get)]
///     pub ndim: usize,
///     #[pyo3(get)]
///     pub description: Option<String>,
///     pub custom_latex: Option<String>,
/// }
/// ```
#[proc_macro_attribute]
pub fn gen_stub_pyclass(_attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_stub::pyclass(item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Embed metadata for Python stub file generation for `#[pyclass]` macro with enum
///
/// ```
/// #[pyo3_stub_gen_derive::gen_stub_pyclass_enum]
/// #[pyo3::pyclass(module = "my_module", name = "DataType")]
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// pub enum PyDataType {
///     #[pyo3(name = "FLOAT")]
///     Float,
///     #[pyo3(name = "INTEGER")]
///     Integer,
/// }
/// ```
#[proc_macro_attribute]
pub fn gen_stub_pyclass_enum(_attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_stub::pyclass_enum(item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Embed metadata for Python stub file generation for `#[pyclass]` macro with a complex enum
///
/// ```
/// #[pyo3_stub_gen_derive::gen_stub_pyclass_complex_enum]
/// #[pyo3::pyclass(module = "my_module", name = "DataType")]
/// #[derive(Debug, Clone)]
/// pub enum PyDataType {
///     #[pyo3(name = "FLOAT")]
///     Float{f: f64},
///     #[pyo3(name = "INTEGER")]
///     Integer(i64),
/// }
/// ```
#[proc_macro_attribute]
pub fn gen_stub_pyclass_complex_enum(_attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_stub::pyclass_complex_enum(item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Embed metadata for Python stub file generation for `#[pymethods]` macro
///
/// ```
/// #[pyo3_stub_gen_derive::gen_stub_pyclass]
/// #[pyo3::pyclass]
/// struct A {}
///
/// #[pyo3_stub_gen_derive::gen_stub_pymethods]
/// #[pyo3::pymethods]
/// impl A {
///     #[getter]
///     fn f(&self) -> Vec<u32> {
///        todo!()
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn gen_stub_pymethods(_attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_stub::pymethods(item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Embed metadata for Python stub file generation for `#[pyfunction]` macro
///
/// ```
/// #[pyo3_stub_gen_derive::gen_stub_pyfunction]
/// #[pyo3::pyfunction]
/// #[pyo3(name = "is_odd")]
/// pub fn is_odd(x: u32) -> bool {
///     todo!()
/// }
/// ```
///
/// The function attributed by `#[gen_stub_pyfunction]` will be appended to default stub file.
/// If you want to append this function to another module, add `module` attribute.
///
/// ```
/// #[pyo3_stub_gen_derive::gen_stub_pyfunction(module = "my_module.experimental")]
/// #[pyo3::pyfunction]
/// #[pyo3(name = "is_odd")]
/// pub fn is_odd(x: u32) -> bool {
///     todo!()
/// }
/// ```
#[proc_macro_attribute]
pub fn gen_stub_pyfunction(attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_stub::pyfunction(attr.into(), item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Do nothing but remove all `#[gen_stub(xxx)]` for `pyclass`, `pymethods`, and `pyfunction`.
///
/// It is useful to use `#[gen_stub(xxx)]` under feature-gating stub-gen.
///
/// E.g., only generate .pyi when `stub-gen` feature is turned-on:
/// ```ignore
/// #[cfg_attr(feature = "stub-gen", pyo3_stub_gen_derive::gen_stub_pymethods)]
/// #[cfg_attr(not(feature = "stub-gen"), pyo3_stub_gen_derive::remove_gen_stub)]
/// #[pymethods]
/// impl A {
///     #[gen_stub(override_return_type(type_repr="typing_extensions.Self", imports=("typing_extensions")))]
///     #[new]
///     pub fn new() -> Self {
///         Self::default()
///     }
/// }
/// #[cfg(feature = "stub-gen")]
/// define_stub_info_gatherer!(stub_info);
/// ```
/// With Cargo.toml:
/// ```toml
/// [features]
/// stub-gen = ["dep:pyo3-stub-gen"]
/// [dependencies]
/// pyo3-stub-gen = {version = "*", optional = true}
/// pyo3-stub-gen-derive = "*"
/// ```
#[proc_macro_attribute]
pub fn remove_gen_stub(_attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_stub::prune_gen_stub(item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Generate PyFunctionInfo from Python stub string
///
/// This proc-macro parses Python stub syntax and generates a PyFunctionInfo structure.
/// It should be used inside `inventory::submit!` blocks.
///
/// ```ignore
/// submit! {
///     gen_function_from_python! {
///         r#"
///             import collections.abc
///             import typing
///
///             def fn_override_type(cb: collections.abc.Callable[[str], typing.Any]) -> collections.abc.Callable[[str], typing.Any]: ...
///         "#
///     }
/// }
/// ```
#[proc_macro]
pub fn gen_function_from_python(input: TokenStream) -> TokenStream {
    gen_stub::gen_function_from_python_impl(input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Generate PyMethodsInfo from Python class definition
///
/// This proc-macro parses Python class definition syntax and generates a PyMethodsInfo structure.
/// It should be used inside `inventory::submit!` blocks.
///
/// Supports single or multiple method definitions within a class:
///
/// ```ignore
/// // Single method
/// submit! {
///     gen_methods_from_python! {
///         r#"
///         class Incrementer:
///             def increment_1(self, x: int) -> int:
///                 """Increment by one"""
///         "#
///     }
/// }
///
/// // Multiple methods
/// submit! {
///     gen_methods_from_python! {
///         r#"
///         class Incrementer2:
///             def increment_2(self, x: float) -> float:
///                 """Increment by two (float version)"""
///
///             def __new__(cls) -> Incrementer2:
///                 """Constructor"""
///
///             def increment_2(self, x: int) -> int:
///                 """Increment by two (int version)"""
///         "#
///     }
/// }
/// ```
#[proc_macro]
pub fn gen_methods_from_python(input: TokenStream) -> TokenStream {
    gen_stub::gen_methods_from_python_impl(input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
