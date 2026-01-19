//! Test case for same-name submodule collision.
//!
//! This tests what happens when two different package hierarchies have
//! submodules with the same name, and a function references types from both.
//!
//! Structure:
//! - same_name_submod.pkg_a.sub_mod.ClassA
//! - same_name_submod.pkg_b.sub_mod.ClassB
//! - same_name_submod.use_both(a: ClassA, b: ClassB) -> references both

use pyo3::prelude::*;
use pyo3_stub_gen::{derive::*, *};

// ClassA in pkg_a.sub_mod
#[gen_stub_pyclass]
#[pyclass]
#[pyo3(module = "same_name_submod.pkg_a.sub_mod")]
#[derive(Clone)]
pub struct ClassA {
    pub value: i32,
}

#[gen_stub_pymethods]
#[pymethods]
impl ClassA {
    #[new]
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

// ClassB in pkg_b.sub_mod
#[gen_stub_pyclass]
#[pyclass]
#[pyo3(module = "same_name_submod.pkg_b.sub_mod")]
#[derive(Clone)]
pub struct ClassB {
    pub value: i32,
}

#[gen_stub_pymethods]
#[pymethods]
impl ClassB {
    #[new]
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

// Function in root module that uses both ClassA and ClassB
// This will cause import collision:
//   from same_name_submod.pkg_a import sub_mod
//   from same_name_submod.pkg_b import sub_mod  # collision!
#[gen_stub_pyfunction]
#[pyfunction]
pub fn use_both(a: ClassA, b: ClassB) -> i32 {
    a.value + b.value
}

#[pymodule]
fn same_name_submod(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(use_both, m)?)?;
    pkg_a(m)?;
    pkg_b(m)?;
    Ok(())
}

fn pkg_a(parent: &Bound<PyModule>) -> PyResult<()> {
    let py = parent.py();
    let pkg = PyModule::new(py, "pkg_a")?;
    let sub = PyModule::new(py, "sub_mod")?;
    sub.add_class::<ClassA>()?;
    pkg.add_submodule(&sub)?;
    parent.add_submodule(&pkg)?;
    Ok(())
}

fn pkg_b(parent: &Bound<PyModule>) -> PyResult<()> {
    let py = parent.py();
    let pkg = PyModule::new(py, "pkg_b")?;
    let sub = PyModule::new(py, "sub_mod")?;
    sub.add_class::<ClassB>()?;
    pkg.add_submodule(&sub)?;
    parent.add_submodule(&pkg)?;
    Ok(())
}

define_stub_info_gatherer!(stub_info);
