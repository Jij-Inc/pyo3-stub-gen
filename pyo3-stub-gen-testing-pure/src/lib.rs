#[cfg_attr(target_os = "macos", doc = include_str!("../../README.md"))]
mod readme {}

use pyo3::{exceptions::PyRuntimeError, prelude::*};
use pyo3_stub_gen::{create_exception, define_stub_info_gatherer, derive::gen_stub_pyfunction};

/// Returns the sum of two numbers as a string.
#[gen_stub_pyfunction]
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

create_exception!(pyo3_stub_gen_testing_pure, MyError, PyRuntimeError);

/// Initializes the Python module
#[pymodule]
fn pyo3_stub_gen_testing_pure(m: &Bound<PyModule>) -> PyResult<()> {
    m.add("MyError", m.py().get_type_bound::<MyError>())?;
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
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
