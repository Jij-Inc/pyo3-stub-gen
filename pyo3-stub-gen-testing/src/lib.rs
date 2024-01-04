use linkme::distributed_slice;
use pyo3::prelude::*;
use pyo3_stub_gen::type_info::*;

/// Returns the sum of two numbers as a string.
///
/// Test of running doc-test
///
/// ```rust
/// assert_eq!(2 + 2, 4);
/// ```
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[distributed_slice(PYFUNCTIONS)]
pub static PYFUNCTION_SUM_AS_STRING: PyFunctionInfo = PyFunctionInfo {
    name: "sum_as_string",
    r#return: no_return_type_output,
    args: &[/* dummy */],
    doc: "",
    signature: None,
    module: None,
};

pub fn dbg() {
    dbg!(PYFUNCTIONS.len());
    for info in PYFUNCTIONS {
        dbg!(info);
    }
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
