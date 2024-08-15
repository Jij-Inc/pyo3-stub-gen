use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::*};

// Specify the module name explicitly
#[gen_stub_pyclass]
#[pyclass(module = "mixed.main_mod")]
#[derive(Debug)]
struct A {
    x: usize,
}

#[gen_stub_pymethods]
#[pymethods]
impl A {
    fn show_x(&self) {
        println!("x = {}", self.x);
    }
}

#[gen_stub_pyfunction(module = "mixed.main_mod")]
#[pyfunction]
fn create_a(x: usize) -> A {
    A { x }
}

// Do not specify the module name explicitly
// This will be placed in the main module
#[gen_stub_pyclass]
#[pyclass]
#[derive(Debug)]
struct B {
    x: usize,
}

#[gen_stub_pymethods]
#[pymethods]
impl B {
    fn show_x(&self) {
        println!("x = {}", self.x);
    }
}

#[gen_stub_pyfunction]
#[pyfunction]
fn create_b(x: usize) -> B {
    B { x }
}

#[pymodule]
fn main_mod(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<A>()?;
    m.add_class::<B>()?;
    m.add_function(wrap_pyfunction!(create_a, m)?)?;
    m.add_function(wrap_pyfunction!(create_b, m)?)?;
    Ok(())
}

define_stub_info_gatherer!(stub_info);

/// Test of unit test for testing link problem
#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert_eq!(2 + 2, 4);
    }
}
