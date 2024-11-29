#[cfg_attr(target_os = "macos", doc = include_str!("../../../README.md"))]
mod readme {}

use ahash::RandomState;
use pyo3::{exceptions::PyRuntimeError, prelude::*, types::*};
use pyo3_stub_gen::{create_exception, define_stub_info_gatherer, derive::*, module_variable};
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

    fn ref_test<'a>(&self, x: Bound<'a, PyDict>) -> Bound<'a, PyDict> {
        x
    }

    async fn async_get_x(&self) -> usize {
        self.x
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

#[gen_stub_pyfunction]
#[pyfunction]
fn echo_path(path: PathBuf) -> PyResult<PathBuf> {
    Ok(path)
}

#[gen_stub_pyfunction]
#[pyfunction]
fn ahash_dict() -> HashMap<String, i32, RandomState> {
    let mut map: HashMap<String, i32, RandomState> = HashMap::with_hasher(RandomState::new());
    map.insert("apple".to_string(), 3);
    map.insert("banana".to_string(), 2);
    map.insert("orange".to_string(), 5);
    map
}

#[gen_stub_pyclass_enum]
#[pyclass]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Number {
    #[pyo3(name = "FLOAT")]
    Float,
    #[pyo3(name = "INTEGER")]
    Integer,
}

module_variable!("pure", "MY_CONSTANT", usize);

#[gen_stub_pyfunction]
#[pyfunction]
async fn async_num() -> i32 {
    123
}

/// Initializes the Python module
#[pymodule]
fn pure(m: &Bound<PyModule>) -> PyResult<()> {
    m.add("MyError", m.py().get_type_bound::<MyError>())?;
    m.add("MY_CONSTANT", 19937)?;
    m.add_class::<A>()?;
    m.add_class::<Number>()?;
    m.add_function(wrap_pyfunction!(sum, m)?)?;
    m.add_function(wrap_pyfunction!(create_dict, m)?)?;
    m.add_function(wrap_pyfunction!(read_dict, m)?)?;
    m.add_function(wrap_pyfunction!(create_a, m)?)?;
    m.add_function(wrap_pyfunction!(str_len, m)?)?;
    m.add_function(wrap_pyfunction!(echo_path, m)?)?;
    m.add_function(wrap_pyfunction!(ahash_dict, m)?)?;
    m.add_function(wrap_pyfunction!(async_num, m)?)?;
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
