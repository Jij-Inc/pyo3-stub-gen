#![allow(clippy::type_complexity)]

//! NumPy integration examples for testing stub generation

use numpy::{
    AllowTypeChange, PyArray1, PyArrayLike1, PyReadonlyArray1, PyReadonlyArrayDyn,
    PyUntypedArrayMethods, TypeMustMatch,
};
use pyo3::prelude::*;
use pyo3_stub_gen::derive::gen_stub_pyfunction;

/// Takes a 1D NumPy array and returns its sum
#[gen_stub_pyfunction]
#[pyfunction]
pub fn sum_array_1d(array: PyReadonlyArray1<f64>) -> f64 {
    array.as_array().sum()
}

/// Creates a new 1D NumPy array filled with zeros
#[gen_stub_pyfunction]
#[pyfunction]
pub fn create_zeros_1d<'py>(py: Python<'py>, size: usize) -> Bound<'py, PyArray1<f64>> {
    PyArray1::zeros(py, size, false)
}

/// Convert integer array to float array
#[gen_stub_pyfunction]
#[pyfunction]
pub fn int_to_float<'py>(
    py: Python<'py>,
    array: PyReadonlyArray1<i32>,
) -> Bound<'py, PyArray1<f64>> {
    let arr = array.as_array();
    let float_arr = arr.mapv(|x| x as f64);
    PyArray1::from_owned_array(py, float_arr)
}

/// Process float32 array
#[gen_stub_pyfunction]
#[pyfunction]
pub fn process_float32_array<'py>(
    py: Python<'py>,
    array: PyReadonlyArray1<f32>,
) -> Bound<'py, PyArray1<f32>> {
    let arr = array.as_array();
    let result = arr.mapv(|x| x * 2.0);
    PyArray1::from_owned_array(py, result)
}

/// Working with dynamic-dimensional arrays
#[gen_stub_pyfunction]
#[pyfunction]
pub fn sum_dynamic_array(array: PyReadonlyArrayDyn<f64>) -> f64 {
    array.as_array().sum()
}

/// Optional array parameter
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (array=None))]
pub fn optional_array_param(array: Option<PyReadonlyArray1<f64>>) -> String {
    match array {
        Some(arr) => format!("Array with {} elements", arr.shape()[0]),
        None => "No array provided".to_string(),
    }
}

/// Return multiple arrays
#[gen_stub_pyfunction]
#[pyfunction]
pub fn split_array<'py>(
    py: Python<'py>,
    array: PyReadonlyArray1<f64>,
) -> PyResult<(Bound<'py, PyArray1<f64>>, Bound<'py, PyArray1<f64>>)> {
    let arr = array.as_array();
    let len = arr.len();

    if len % 2 != 0 {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "Array length must be even for splitting",
        ));
    }

    let mid = len / 2;
    let first_half = arr.slice(numpy::ndarray::s![..mid]).to_owned();
    let second_half = arr.slice(numpy::ndarray::s![mid..]).to_owned();

    Ok((
        PyArray1::from_owned_array(py, first_half),
        PyArray1::from_owned_array(py, second_half),
    ))
}

/// Count true values in boolean array
#[gen_stub_pyfunction]
#[pyfunction]
pub fn count_true(array: PyReadonlyArray1<bool>) -> usize {
    array.as_array().iter().filter(|&&x| x).count()
}

/// PyArrayLike with AllowTypeChange - accepts any type that numpy.asarray accepts
#[gen_stub_pyfunction]
#[pyfunction]
pub fn np_allow_type_change<'py>(
    py: Python<'py>,
    x: PyArrayLike1<'py, f64, AllowTypeChange>,
) -> Bound<'py, PyArray1<f64>> {
    PyArray1::<f64>::from_array(py, &x.as_array())
}

/// PyArrayLike with TypeMustMatch - requires exact type match
#[gen_stub_pyfunction]
#[pyfunction]
pub fn np_type_must_match<'py>(
    py: Python<'py>,
    x: PyArrayLike1<'py, i16, TypeMustMatch>,
) -> Bound<'py, PyArray1<i16>> {
    PyArray1::<i16>::from_array(py, &x.as_array())
}
