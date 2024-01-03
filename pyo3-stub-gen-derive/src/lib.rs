mod gen_stub;

use proc_macro::TokenStream;

/// Embed metadata for Python stub file generation for `#[pyclass]` macro
///
/// ```
/// # use pyo3_stub_gen::*;
/// # use pyo3::*;
/// #[gen_stub_pyclass]
/// #[pyclass(mapping, module = "my_module", name = "Placeholder")]
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
/// # use pyo3_stub_gen::*;
/// # use pyo3::*;
/// #[gen_stub_pyclass_enum]
/// #[pyclass(module = "my_module", name = "DataType")]
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

/// Embed metadata for Python stub file generation for `#[pymethods]` macro
///
/// ```
/// # use pyo3_stub_gen::*;
/// # use pyo3::*;
/// # #[gen_stub_pyclass]
/// # #[pyclass]
/// # struct PyAddOp {}
/// # #[pyclass]
/// # struct Expression {}
/// #[gen_stub_pymethods]
/// #[pymethods]
/// impl PyAddOp {
///     #[getter]
///     fn get_terms(&self) -> Vec<Expression> {
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
/// # use pyo3_stub_gen::*;
/// # use pyo3::*;
/// # #[pyclass]
/// # #[derive(Clone)]
/// # pub struct Expression {}
/// #[gen_stub_pyfunction]
/// #[pyfunction]
/// #[pyo3(name = "is_linear")]
/// pub fn py_is_linear(expr: Expression) -> bool {
///     todo!()
/// }
/// ```
///
/// The function attributed by `#[gen_stub_pyfunction]` will be appended to default stub file.
/// If you want to append this function to another module, add `module` attribute.
///
/// ```
/// # use pyo3_stub_gen::*;
/// # use pyo3::*;
/// # #[pyclass]
/// # #[derive(Clone)]
/// # pub struct Expression {}
/// #[gen_stub_pyfunction(module = "my_module.experimental")]
/// #[pyfunction]
/// #[pyo3(name = "is_linear")]
/// pub fn py_is_linear(expr: Expression) -> bool {
///     todo!()
/// }
/// ```
#[proc_macro_attribute]
pub fn gen_stub_pyfunction(attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_stub::pyfunction(attr.into(), item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
