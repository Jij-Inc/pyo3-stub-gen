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
#[pyclass(extends=PyDate)]
struct MyDate;

#[gen_stub_pyclass]
#[pyclass(subclass)]
#[derive(Debug)]
struct A {
    #[gen_stub(default = A::default().x)]
    #[pyo3(get, set)]
    x: usize,
}

impl Default for A {
    fn default() -> Self {
        Self { x: 2 }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl A {
    /// This is a constructor of :class:`A`.
    #[new]
    fn new(x: usize) -> Self {
        Self { x }
    }
    /// class attribute NUM1
    #[classattr]
    const NUM1: usize = 2;
    /// class attribute NUM2
    #[expect(non_snake_case)]
    #[classattr]
    fn NUM2() -> usize {
        2
    }
    fn show_x(&self) {
        println!("x = {}", self.x);
    }

    fn ref_test<'a>(&self, x: Bound<'a, PyDict>) -> Bound<'a, PyDict> {
        x
    }

    #[gen_stub(skip)]
    fn need_skip(&self) {}
}

#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (x = 2))]
fn create_a(x: usize) -> A {
    A { x }
}

#[gen_stub_pyclass]
#[pyclass(extends=A)]
#[derive(Debug)]
struct B;

/// `C` only impl `FromPyObject`
#[derive(Debug)]
struct C {
    x: usize,
}
#[gen_stub_pyfunction]
#[pyfunction(signature = (c=None))]
fn print_c(c: Option<C>) {
    if let Some(c) = c {
        println!("{}", c.x);
    } else {
        println!("None");
    }
}
impl FromPyObject<'_> for C {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        Ok(C { x: ob.extract()? })
    }
}
impl pyo3_stub_gen::PyStubType for C {
    fn type_output() -> pyo3_stub_gen::TypeInfo {
        usize::type_output()
    }
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
#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Number {
    #[pyo3(name = "FLOAT")]
    Float,
    #[pyo3(name = "INTEGER")]
    Integer,
}

#[gen_stub_pyclass_enum]
#[pyclass(eq, eq_int)]
#[pyo3(rename_all = "UPPERCASE")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NumberRenameAll {
    /// Float variant
    Float,
    Integer,
}

#[gen_stub_pymethods]
#[pymethods]
impl Number {
    #[getter]
    /// Whether the number is a float.
    fn is_float(&self) -> bool {
        matches!(self, Self::Float)
    }

    #[getter]
    /// Whether the number is an integer.
    fn is_integer(&self) -> bool {
        matches!(self, Self::Integer)
    }
}

module_variable!("pure", "MY_CONSTANT", usize);

// Test if non-any PyObject Target can be a default value
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (num = Number::Float))]
fn default_value(num: Number) -> Number {
    num
}

/// Initializes the Python module
#[pymodule]
fn pure(m: &Bound<PyModule>) -> PyResult<()> {
    m.add("MyError", m.py().get_type::<MyError>())?;
    m.add("MY_CONSTANT", 19937)?;
    m.add_class::<MyDate>()?;
    m.add_class::<A>()?;
    m.add_class::<B>()?;
    m.add_class::<Number>()?;
    m.add_class::<NumberRenameAll>()?;
    m.add_function(wrap_pyfunction!(sum, m)?)?;
    m.add_function(wrap_pyfunction!(create_dict, m)?)?;
    m.add_function(wrap_pyfunction!(read_dict, m)?)?;
    m.add_function(wrap_pyfunction!(create_a, m)?)?;
    m.add_function(wrap_pyfunction!(print_c, m)?)?;
    m.add_function(wrap_pyfunction!(str_len, m)?)?;
    m.add_function(wrap_pyfunction!(echo_path, m)?)?;
    m.add_function(wrap_pyfunction!(ahash_dict, m)?)?;
    m.add_function(wrap_pyfunction!(default_value, m)?)?;
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
