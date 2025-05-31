use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::*};

#[gen_stub_pyfunction(module = "mixed_sub_multiple.main_mod.mod_a")]
#[pyfunction(name = "greet_a")]
pub fn greet_a(kind: GreetingEnum) {
    println!("Hello from mod_A! (kind = ${kind:?}");
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

#[gen_stub_pyclass_enum]
#[pyclass(eq, eq_int, module = "mixed_sub_multiple.main_mod.mod_b")]
#[pyo3(rename_all = "UPPERCASE")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GreetingEnum {
    GreetA,
    GreetB,
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
    sub.add_class::<GreetingEnum>()?;
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
