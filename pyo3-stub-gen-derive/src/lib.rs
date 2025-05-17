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
/// #[pyo3_stub_gen_derive::gen_stub_pyclass_rich_enum]
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
pub fn gen_stub_pyclass_rich_enum(_attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_stub::pyclass_rich_enum(item.into())
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
