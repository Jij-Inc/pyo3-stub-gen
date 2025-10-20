use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::*};

// Classes that can be cross-referenced between modules (from mixed_sub_import_type)
#[gen_stub_pyclass]
#[pyclass(module = "mixed_sub.main_mod")]
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

#[gen_stub_pyfunction(module = "mixed_sub.main_mod")]
#[pyfunction]
fn create_a(x: usize) -> A {
    A { x }
}

// Class without explicit module specification
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

// Original functions from mixed_sub
#[gen_stub_pyfunction(module = "mixed_sub.main_mod.mod_a")]
#[pyfunction(name = "greet_a")]
pub fn greet_a() {
    println!("Hello from mod_A!")
}

#[gen_stub_pyfunction(module = "mixed_sub.main_mod")]
#[pyfunction(name = "greet_main")]
pub fn greet_main() {
    println!("Hello from main_mod!")
}

#[gen_stub_pyfunction(module = "mixed_sub.main_mod.mod_b")]
#[pyfunction(name = "greet_b")]
pub fn greet_b() {
    println!("Hello from mod_B!")
}

// Class C in mod_a that references A and B (demonstrates cross-module type references)
#[gen_stub_pyclass]
#[pyclass(module = "mixed_sub.main_mod.mod_a")]
#[derive(Debug)]
struct C {
    a: A,
    b: B,
}

#[gen_stub_pymethods]
#[pymethods]
impl C {
    fn show_x(&self) {
        println!("a.x:");
        self.a.show_x();
        println!("b.x:");
        self.b.show_x();
    }
}

#[gen_stub_pyfunction(module = "mixed_sub.main_mod.mod_a")]
#[pyfunction]
fn create_c(a: A, b: B) -> C {
    C { a, b }
}

// Simple class in mod_b (from mixed_sub)
#[gen_stub_pyclass]
#[pyclass(module = "mixed_sub.main_mod.mod_b")]
#[derive(Debug)]
struct D {
    x: usize,
}

#[gen_stub_pymethods]
#[pymethods]
impl D {
    fn show_x(&self) {
        println!("x = {}", self.x);
    }
}

#[gen_stub_pyfunction(module = "mixed_sub.main_mod.mod_b")]
#[pyfunction]
fn create_d(x: usize) -> D {
    D { x }
}

// Function in int submodule to test namespace collision
#[gen_stub_pyfunction(module = "mixed_sub.main_mod.int")]
#[pyfunction]
fn dummy_int_fun(x: usize) -> usize {
    x
}

// Test function with both module and python parameters (bug reproduction case)
// This should be placed in mod_a submodule
#[gen_stub_pyfunction(
    module = "mixed_sub.main_mod.mod_a",
    python = r#"
    import typing

    def test_module_with_python(x: typing.Generator[int, None, None]) -> int:
        """Test function with both module and python parameters"""
"#
)]
#[pyfunction]
fn test_module_with_python(_x: &Bound<PyAny>) -> PyResult<usize> {
    Ok(42)
}

#[pymodule]
fn main_mod(m: &Bound<PyModule>) -> PyResult<()> {
    // Add classes and functions to main module
    m.add_class::<A>()?;
    m.add_class::<B>()?;
    m.add_function(wrap_pyfunction!(create_a, m)?)?;
    m.add_function(wrap_pyfunction!(create_b, m)?)?;
    m.add_function(wrap_pyfunction!(greet_main, m)?)?;

    // Add submodules
    mod_a(m)?;
    mod_b(m)?;
    int_mod(m)?;
    Ok(())
}

fn mod_a(parent: &Bound<PyModule>) -> PyResult<()> {
    let py = parent.py();
    let sub = PyModule::new(py, "mod_a")?;
    sub.add_class::<C>()?;
    sub.add_function(wrap_pyfunction!(greet_a, &sub)?)?;
    sub.add_function(wrap_pyfunction!(create_c, &sub)?)?;
    sub.add_function(wrap_pyfunction!(test_module_with_python, &sub)?)?;
    parent.add_submodule(&sub)?;
    Ok(())
}

fn mod_b(parent: &Bound<PyModule>) -> PyResult<()> {
    let py = parent.py();
    let sub = PyModule::new(py, "mod_b")?;
    sub.add_class::<D>()?;
    sub.add_function(wrap_pyfunction!(greet_b, &sub)?)?;
    sub.add_function(wrap_pyfunction!(create_d, &sub)?)?;
    parent.add_submodule(&sub)?;
    Ok(())
}

/// A dummy module to test namespace collision with built-in 'int'
fn int_mod(parent: &Bound<PyModule>) -> PyResult<()> {
    let py = parent.py();
    let sub = PyModule::new(py, "int")?;
    sub.add_function(wrap_pyfunction!(dummy_int_fun, &sub)?)?;
    parent.add_submodule(&sub)?;
    Ok(())
}

// Test gen_function_from_python! with module parameter
use pyo3_stub_gen::inventory::submit;

submit! {
    gen_function_from_python! {
        module = "mixed_sub.main_mod.mod_b",
        r#"
        import typing

        def test_submit_with_module(values: typing.List[int]) -> int:
            """Test function defined with gen_function_from_python! and module parameter"""
        "#
    }
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
