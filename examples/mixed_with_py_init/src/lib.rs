//! Example demonstrating __init__.py and __init__.pyi coexistence problem
//!
//! This example shows the issue where:
//! 1. User has `__init__.py` with re-exports from both Python and Rust modules
//! 2. pyo3-stub-gen generates `__init__.pyi` with only Rust types
//! 3. Type checkers prioritize `.pyi` over `.py`, so Python re-exports become invisible

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

/// The native module that gets imported by __init__.py
#[pymodule]
fn _native(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<NativeClass>()?;
    m.add_function(wrap_pyfunction!(native_function, m)?)?;
    Ok(())
}

define_stub_info_gatherer!(stub_info);
