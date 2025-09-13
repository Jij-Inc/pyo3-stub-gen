use numpy::{AllowTypeChange, PyArray1, PyArrayLike1, TypeMustMatch};
use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::*};


#[gen_stub_pyfunction]
#[pyfunction]
fn np_allow_type_change<'py>(py: Python<'py>, x: PyArrayLike1<'py, f64, AllowTypeChange>) -> Bound<'py, PyArray1<f64>> {
    PyArray1::<f64>::from_array(py, &x.as_array())
}

#[gen_stub_pyfunction]
#[pyfunction]
fn np_type_must_match<'py>(py: Python<'py>, x: PyArrayLike1<'py, i16, TypeMustMatch>) -> Bound<'py, PyArray1<i16>> {
    PyArray1::<i16>::from_array(py, &x.as_array())
}

#[pymodule]
fn numpy_example(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(np_allow_type_change, m)?)?;
    m.add_function(wrap_pyfunction!(np_type_must_match, m)?)?;
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
