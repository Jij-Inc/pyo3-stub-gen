use pyo3::prelude::*;
use pyo3_stub_gen::{derive::*, *};

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct ClassA {}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct ClassB {}

#[gen_stub_pymethods]
#[pymethods]
impl ClassB {
    #[allow(non_snake_case)]
    pub fn ClassA(&self) -> ClassA {
        ClassA {}
    }

    pub fn collision(&self, a: ClassA) -> ClassA {
        a
    }
}

#[pymodule]
fn main_mod(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<ClassA>()?;
    m.add_class::<ClassB>()?;
    Ok(())
}

define_stub_info_gatherer!(stub_info);
