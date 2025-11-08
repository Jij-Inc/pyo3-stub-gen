use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::*};

#[gen_stub_pyfunction(module = "empty_super_module.sub_mod")]
#[pyfunction(name = "greet")]
pub fn greet() {
    println!("Hello from sub_mod!")
}

#[gen_stub_pyfunction(module = "empty_super_module.deep.nested.module")]
#[pyfunction(name = "deep_function")]
pub fn deep_function() {
    println!("Hello from deep nested module!")
}

#[pymodule]
fn main_mod(m: &Bound<PyModule>) -> PyResult<()> {
    sub_mod(m)?;
    deep_nested_mod(m)?;
    Ok(())
}

fn sub_mod(parent: &Bound<PyModule>) -> PyResult<()> {
    let py = parent.py();
    let sub = PyModule::new(py, "sub_mod")?;
    sub.add_function(wrap_pyfunction!(greet, &sub)?)?;
    parent.add_submodule(&sub)?;
    Ok(())
}

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

/// Test of unit test for testing link problem
#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert_eq!(2 + 2, 4);
    }
}
