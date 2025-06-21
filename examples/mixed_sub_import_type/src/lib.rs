use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::*};

// Specify the module name explicitly
#[gen_stub_pyclass]
#[pyclass(module = "mixed_sub_import_type.main_mod")]
#[derive(Debug, Clone)]
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

#[gen_stub_pyfunction(module = "mixed_sub_import_type.main_mod")]
#[pyfunction]
fn create_a(x: usize) -> A {
    A { x }
}

// Do not specify the module name explicitly
// This will be placed in the main module
#[gen_stub_pyclass]
#[pyclass]
#[derive(Debug, Clone)]
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

// Class in submodule
#[gen_stub_pyclass]
#[pyclass(module = "mixed_sub_import_type.main_mod.sub_mod")]
#[derive(Debug)]
struct C {
    a: A,
    b: B
}

#[gen_stub_pymethods]
#[pymethods]
impl C {
    fn show_x(&self) {
        println!("a.x");
        self.a.show_x();
        println!("b.x");
        self.b.show_x()
    }
}

#[gen_stub_pyfunction(module = "mixed_sub_import_type.main_mod.sub_mod")]
#[pyfunction]
fn create_c(a: A, b: B) -> C {
    C { a, b }
}

#[gen_stub_pyfunction(module = "mixed_sub_import_type.main_mod.int")]
#[pyfunction]
fn dummy_int_fun(x: usize) -> usize {
    x
}

#[pymodule]
fn main_mod(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<A>()?;
    m.add_class::<B>()?;
    m.add_function(wrap_pyfunction!(create_a, m)?)?;
    m.add_function(wrap_pyfunction!(create_b, m)?)?;
    sub_mod(m)?;
    int_mod(m)?;
    Ok(())
}

fn sub_mod(parent: &Bound<PyModule>) -> PyResult<()> {
    let py = parent.py();
    let sub = PyModule::new(py, "sub_mod")?;
    sub.add_class::<C>()?;
    sub.add_function(wrap_pyfunction!(create_c, &sub)?)?;
    parent.add_submodule(&sub)?;
    Ok(())
}

/// A dummy module to pollute namespace with unqualified `int`
fn int_mod(parent: &Bound<PyModule>) -> PyResult<()> {
    let py = parent.py();
    let sub = PyModule::new(py, "int")?;
    sub.add_function(wrap_pyfunction!(dummy_int_fun, &sub)?)?;
    parent.add_submodule(&sub)?;
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
