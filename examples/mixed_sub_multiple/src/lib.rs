use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::*};

#[gen_stub_pyfunction(module = "mixed_sub_multiple.main_mod.mod_a")]
#[pyfunction(name = "greet_a")]
pub fn greet_a() {
    println!("Hello from mod_A!")
}

#[gen_stub_pyfunction(module = "mixed_sub_multiple.main_mod")]
#[pyfunction(name = "greet_main")]
pub fn greet_main() {
    println!("Hello from main_mod!")
}

#[gen_stub_pyfunction(module = "mixed_sub_multiple.main_mod.mod_b")]
#[pyfunction(name = "greet_b")]
pub fn greet_b() {
    println!("Hello from mod_B!")
}

#[pymodule]
fn main_mod(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(greet_main, m)?)?;
    mod_a(m)?;
    mod_b(m)?;
    Ok(())
}

#[pymodule]
fn mod_a(parent: &Bound<PyModule>) -> PyResult<()> {
    let py = parent.py();
    let sub = PyModule::new(py, "mod_a")?;
    sub.add_function(wrap_pyfunction!(greet_a, &sub)?)?;
    parent.add_submodule(&sub)?;
    Ok(())
}

#[pymodule]
fn mod_b(parent: &Bound<PyModule>) -> PyResult<()> {
    let py = parent.py();
    let sub = PyModule::new(py, "mod_b")?;
    sub.add_function(wrap_pyfunction!(greet_b, &sub)?)?;
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
