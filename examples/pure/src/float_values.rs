//! Test f64 special values (INFINITY, NEG_INFINITY, NAN) as default values and constants

use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

/// A class to test f64 special values
#[gen_stub_pyclass]
#[pyclass]
pub struct FloatValues;

#[gen_stub_pymethods]
#[pymethods]
impl FloatValues {
    /// Class attribute: positive infinity
    #[classattr]
    const POSITIVE_INF: f64 = f64::INFINITY;

    /// Class attribute: negative infinity
    #[classattr]
    const NEGATIVE_INF: f64 = f64::NEG_INFINITY;

    /// Class attribute: NaN
    #[classattr]
    const NAN_VALUE: f64 = f64::NAN;

    /// Class attribute: regular float
    #[classattr]
    const REGULAR_FLOAT: f64 = 3.14;

    #[new]
    fn new() -> Self {
        FloatValues
    }
}

/// Function with infinity as default value
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (threshold = f64::INFINITY))]
pub fn with_infinity_default(threshold: f64) -> f64 {
    threshold
}

/// Function with negative infinity as default value
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (threshold = f64::NEG_INFINITY))]
pub fn with_neg_infinity_default(threshold: f64) -> f64 {
    threshold
}

/// Function with NaN as default value
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (value = f64::NAN))]
pub fn with_nan_default(value: f64) -> f64 {
    value
}

/// Function with regular float default value
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (value = 1.5))]
pub fn with_float_default(value: f64) -> f64 {
    value
}
