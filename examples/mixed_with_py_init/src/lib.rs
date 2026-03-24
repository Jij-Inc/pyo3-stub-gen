//! Example demonstrating __init__.py and __init__.pyi coexistence problem
//!
//! This example shows:
//! 1. User has `__init__.py` with re-exports from both Python and Rust modules
//! 2. pyo3-stub-gen generates `__init__.pyi` only for PyO3-generated modules
//! 3. Pure Python parent modules keep their `__init__.py` without being shadowed
//!
//! It also demonstrates:
//! - Deep nested submodules via `add_submodule`
//! - The `module` parameter in `gen_stub_*` should match the actual runtime path
//!   determined by the `#[pymodule]` entry point and `add_submodule` calls

use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::*};

/// A class defined in Rust that will be re-exported from __init__.py
#[gen_stub_pyclass]
#[pyclass(module = "mixed_with_py_init._native")]
#[derive(Debug, Clone)]
pub struct NativeClass {
    #[pyo3(get, set)]
    value: i32,
}

#[gen_stub_pymethods]
#[pymethods]
impl NativeClass {
    #[new]
    fn new(value: i32) -> Self {
        NativeClass { value }
    }

    fn double(&self) -> i32 {
        self.value * 2
    }
}

/// A function defined in Rust
#[gen_stub_pyfunction(module = "mixed_with_py_init._native")]
#[pyfunction]
pub fn native_function(x: i32) -> i32 {
    x + 1
}

/// A function in a deeply nested submodule.
/// The module path must match the actual runtime structure:
/// _native -> deep -> nested -> module
#[gen_stub_pyfunction(module = "mixed_with_py_init._native.deep.nested.module")]
#[pyfunction]
pub fn deep_function() -> String {
    "Hello from deep nested module!".to_string()
}

/// The native module that gets imported by __init__.py
#[pymodule]
fn _native(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<NativeClass>()?;
    m.add_function(wrap_pyfunction!(native_function, m)?)?;

    // Add deeply nested submodule structure
    deep_nested_mod(m)?;

    Ok(())
}

/// Creates the deep.nested.module submodule hierarchy
fn deep_nested_mod(parent: &Bound<PyModule>) -> PyResult<()> {
    let py = parent.py();
    let deep = PyModule::new(py, "deep")?;
    let nested = PyModule::new(py, "nested")?;
    let module = PyModule::new(py, "module")?;

    module.add_function(wrap_pyfunction!(deep_function, &module)?)?;
    nested.add_submodule(&module)?;
    deep.add_submodule(&nested)?;
    parent.add_submodule(&deep)?;
    Ok(())
}

define_stub_info_gatherer!(stub_info);
