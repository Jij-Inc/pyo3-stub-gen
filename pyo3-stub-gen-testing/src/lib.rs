use pyo3::prelude::*;

#[cfg(feature = "stub_gen")]
use pyo3_stub_gen::derive::*;

/// Returns the sum of two numbers as a string.
///
/// Test of running doc-test
///
/// ```rust
/// assert_eq!(2 + 2, 4);
/// ```
#[cfg_attr(feature = "stub_gen", gen_stub_pyfunction)]
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pymodule]
fn pyo3_stub_gen_testing(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert_eq!(2 + 2, 4);
    }
}
