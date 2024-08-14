#[cfg_attr(target_os = "macos", doc = include_str!("../../README.md"))]
mod readme {}

use pyo3::{exceptions::PyRuntimeError, prelude::*};
use pyo3_stub_gen::{create_exception, define_stub_info_gatherer, derive::gen_stub_pyfunction};
use std::collections::HashMap;

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

create_exception!(pyo3_stub_gen_testing_pure, MyError, PyRuntimeError);

/// Initializes the Python module
#[pymodule]
fn pyo3_stub_gen_testing_pure(m: &Bound<PyModule>) -> PyResult<()> {
    m.add("MyError", m.py().get_type_bound::<MyError>())?;
    m.add_function(wrap_pyfunction!(sum, m)?)?;
    m.add_function(wrap_pyfunction!(create_dict, m)?)?;
    m.add_function(wrap_pyfunction!(read_dict, m)?)?;
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
