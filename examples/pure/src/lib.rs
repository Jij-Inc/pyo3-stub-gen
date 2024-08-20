#[cfg_attr(target_os = "macos", doc = include_str!("../../../README.md"))]
mod readme {}

use pyo3::{exceptions::PyRuntimeError, prelude::*};
use pyo3_stub_gen::{create_exception, define_stub_info_gatherer, derive::*};
use std::{collections::HashMap, path::PathBuf};

/// Returns the sum of two numbers as a string.
#[gen_stub_pyfunction]
#[pyfunction]
fn sum(v: Vec<u32>) -> u32 {
    v.iter().sum()
}

#[gen_stub_pyfunction]
#[pyfunction]
fn read_dict(dict: HashMap<usize, HashMap<usize, usize>>) {
    for (k, v) in dict {
        for (k2, v2) in v {
            println!("{} {} {}", k, k2, v2);
        }
    }
}

#[gen_stub_pyfunction]
#[pyfunction]
fn create_dict(n: usize) -> HashMap<usize, Vec<usize>> {
    let mut dict = HashMap::new();
    for i in 0..n {
        dict.insert(i, (0..i).collect());
    }
    dict
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Debug)]
struct A {
    #[pyo3(get, set)]
    x: usize,
}

#[gen_stub_pymethods]
#[pymethods]
impl A {
    #[new]
    fn new(x: usize) -> Self {
        Self { x }
    }

    fn show_x(&self) {
        println!("x = {}", self.x);
    }
}

#[gen_stub_pyfunction]
#[pyfunction]
fn create_a(x: usize) -> A {
    A { x }
}

create_exception!(pure, MyError, PyRuntimeError);

/// Returns the length of the string.
#[gen_stub_pyfunction]
#[pyfunction]
fn str_len(x: &str) -> PyResult<usize> {
    Ok(x.len())
}

#[pyfunction]
fn create_path() -> PyResult<PathBuf> {
    Ok("path".into())
}

/// Initializes the Python module
#[pymodule]
fn pure(m: &Bound<PyModule>) -> PyResult<()> {
    m.add("MyError", m.py().get_type_bound::<MyError>())?;
    m.add_class::<A>()?;
    m.add_function(wrap_pyfunction!(sum, m)?)?;
    m.add_function(wrap_pyfunction!(create_dict, m)?)?;
    m.add_function(wrap_pyfunction!(read_dict, m)?)?;
    m.add_function(wrap_pyfunction!(create_a, m)?)?;
    m.add_function(wrap_pyfunction!(str_len, m)?)?;
    m.add_function(wrap_pyfunction!(create_path, m)?)?;
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
