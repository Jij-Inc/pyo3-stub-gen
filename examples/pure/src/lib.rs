#![allow(deprecated)]

mod custom_exceptions;
mod manual_submit;
mod overloading;
mod overriding;
mod rust_type_marker;

use custom_exceptions::*;
use manual_submit::*;
use overloading::*;
use overriding::*;
use rust_type_marker::*;

#[cfg_attr(target_os = "macos", doc = include_str!("../../../README.md"))]
mod readme {}

use ahash::RandomState;
use pyo3::{prelude::*, types::*};
use pyo3_stub_gen::{define_stub_info_gatherer, derive::*, module_doc, module_variable};
use rust_decimal::Decimal;
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
            println!("{k} {k2} {v2}");
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

/// Add two decimal numbers with high precision
#[gen_stub_pyfunction]
#[pyfunction]
fn add_decimals(a: Decimal, b: Decimal) -> Decimal {
    a + b
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

    #[pyo3(get)]
    y: usize,
}

impl Default for A {
    fn default() -> Self {
        Self { x: 2, y: 10 }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl A {
    /// This is a constructor of :class:`A`.
    #[new]
    fn new(x: usize) -> Self {
        Self { x, y: 10 }
    }
    /// class attribute NUM1
    #[classattr]
    #[pyo3(name = "NUM")]
    const NUM1: usize = 2;

    /// deprecated class attribute NUM3 (will show warning)
    #[deprecated(since = "1.0.0", note = "This constant is deprecated")]
    #[classattr]
    const NUM3: usize = 3;
    /// class attribute NUM2
    #[expect(non_snake_case)]
    #[classattr]
    fn NUM2() -> usize {
        2
    }
    #[classmethod]
    fn classmethod_test1(cls: &Bound<'_, PyType>) {
        _ = cls;
    }

    #[deprecated(since = "1.0.0", note = "This classmethod is deprecated")]
    #[classmethod]
    fn deprecated_classmethod(cls: &Bound<'_, PyType>) {
        _ = cls;
    }

    #[classmethod]
    fn classmethod_test2(_: &Bound<'_, PyType>) {}

    fn show_x(&self) {
        println!("x = {}", self.x);
    }

    fn ref_test<'a>(&self, x: Bound<'a, PyDict>) -> Bound<'a, PyDict> {
        x
    }

    async fn async_get_x(&self) -> usize {
        self.x
    }

    #[gen_stub(skip)]
    fn need_skip(&self) {}

    #[deprecated(since = "1.0.0", note = "This method is deprecated")]
    fn deprecated_method(&self) {
        println!("This method is deprecated");
    }

    #[deprecated(since = "1.0.0", note = "This method is deprecated")]
    #[getter]
    fn deprecated_getter(&self) -> usize {
        self.x
    }

    #[deprecated(since = "1.0.0", note = "This setter is deprecated")]
    #[setter]
    fn set_y(&mut self, value: usize) {
        self.y = value;
    }

    #[deprecated(since = "1.0.0", note = "This staticmethod is deprecated")]
    #[staticmethod]
    fn deprecated_staticmethod() -> usize {
        42
    }
}

#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (x = 2))]
fn create_a(x: usize) -> A {
    A { x, y: 10 }
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

#[gen_stub_pyclass_complex_enum]
#[pyclass]
#[pyo3(rename_all = "UPPERCASE")]
#[derive(Debug, Clone)]
pub enum NumberComplex {
    /// Float variant
    Float(f64),
    /// Integer variant
    #[pyo3(constructor = (int=2))]
    Integer {
        /// The integer value
        int: i32,
    },
}

/// Example from PyO3 documentation for complex enum
/// https://pyo3.rs/v0.25.1/class.html#complex-enums
#[gen_stub_pyclass_complex_enum]
#[pyclass]
enum Shape1 {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    RegularPolygon(u32, f64),
    Nothing {},
}

/// Example from PyO3 documentation for complex enum
/// https://pyo3.rs/v0.25.1/class.html#complex-enums
#[gen_stub_pyclass_complex_enum]
#[pyclass]
enum Shape2 {
    #[pyo3(constructor = (radius=1.0))]
    Circle {
        radius: f64,
    },
    #[pyo3(constructor = (*, width, height))]
    Rectangle {
        width: f64,
        height: f64,
    },
    #[pyo3(constructor = (side_count, radius=1.0))]
    RegularPolygon {
        side_count: u32,
        radius: f64,
    },
    Nothing {},
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

#[gen_stub_pyclass]
#[pyclass]
pub struct DecimalHolder {
    #[pyo3(get)]
    value: Decimal,
}

#[gen_stub_pymethods]
#[pymethods]
impl DecimalHolder {
    #[new]
    fn new(value: Decimal) -> Self {
        Self { value }
    }
}

module_variable!("pure", "MY_CONSTANT1", usize);
module_variable!("pure", "MY_CONSTANT2", usize, 123);

#[gen_stub_pyfunction]
#[pyfunction]
async fn async_num() -> i32 {
    123
}

#[gen_stub_pyfunction]
#[pyfunction]
#[deprecated(since = "1.0.0", note = "This function is deprecated")]
fn deprecated_function() {
    println!("This function is deprecated");
}

// Test if non-any PyObject Target can be a default value
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (num = Number::Float))]
fn default_value(num: Number) -> Number {
    num
}

// These are the tests to test the treatment of `*args` and `**kwargs` in functions

/// Test struct for eq and ord comparison methods
#[gen_stub_pyclass]
#[pyclass(eq, ord)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ComparableStruct {
    #[pyo3(get)]
    pub value: i32,
}

#[gen_stub_pymethods]
#[pymethods]
impl ComparableStruct {
    #[new]
    fn new(value: i32) -> Self {
        Self { value }
    }
}

/// Test struct for hash and str methods
#[gen_stub_pyclass]
#[pyclass(eq, hash, frozen, str)]
#[derive(Debug, Clone, Hash, PartialEq)]
pub struct HashableStruct {
    #[pyo3(get)]
    pub name: String,
}

impl std::fmt::Display for HashableStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HashableStruct({})", self.name)
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl HashableStruct {
    #[new]
    fn new(name: String) -> Self {
        Self { name }
    }
}

/// Takes a variable number of arguments and returns their string representation.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (*args))]
fn func_with_star_arg_typed(
    #[gen_stub(override_type(type_repr = "str"))] args: &Bound<PyTuple>,
) -> String {
    args.to_string()
}

/// Takes a variable number of arguments and returns their string representation.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (*args))]
fn func_with_star_arg(args: &Bound<PyTuple>) -> String {
    args.to_string()
}

/// Takes a variable number of keyword arguments and does nothing
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (**kwargs))]
fn func_with_kwargs(kwargs: Option<&Bound<PyDict>>) -> bool {
    kwargs.is_some()
}

module_doc!("pure", "Document for {} ...", env!("CARGO_PKG_NAME"));

/// Initializes the Python module
#[pymodule]
fn pure(m: &Bound<PyModule>) -> PyResult<()> {
    m.add("MY_CONSTANT1", 19937)?;
    m.add("MY_CONSTANT2", 123)?;
    m.add_class::<A>()?;
    m.add_class::<B>()?;
    m.add_class::<MyDate>()?;
    m.add_class::<Number>()?;
    m.add_class::<NumberRenameAll>()?;
    m.add_class::<NumberComplex>()?;
    m.add_class::<Shape1>()?;
    m.add_class::<Shape2>()?;
    m.add_class::<Incrementer>()?;
    m.add_class::<Incrementer2>()?;
    m.add_class::<OverrideType>()?;
    m.add_class::<ComparableStruct>()?;
    m.add_class::<HashableStruct>()?;
    m.add_class::<DecimalHolder>()?;
    m.add_class::<DataContainer>()?;
    m.add_class::<Placeholder>()?;
    m.add_class::<Calculator>()?;
    m.add_class::<InstanceValue>()?;
    m.add_class::<Problem>()?;
    m.add_function(wrap_pyfunction!(sum, m)?)?;
    m.add_function(wrap_pyfunction!(create_dict, m)?)?;
    m.add_function(wrap_pyfunction!(read_dict, m)?)?;
    m.add_function(wrap_pyfunction!(create_a, m)?)?;
    m.add_function(wrap_pyfunction!(print_c, m)?)?;
    m.add_function(wrap_pyfunction!(str_len, m)?)?;
    m.add_function(wrap_pyfunction!(echo_path, m)?)?;
    m.add_function(wrap_pyfunction!(ahash_dict, m)?)?;
    m.add_function(wrap_pyfunction!(async_num, m)?)?;
    m.add_function(wrap_pyfunction!(deprecated_function, m)?)?;
    m.add_function(wrap_pyfunction!(default_value, m)?)?;
    m.add_function(wrap_pyfunction!(fn_override_type, m)?)?;
    m.add_function(wrap_pyfunction!(fn_with_python_param, m)?)?;
    m.add_function(wrap_pyfunction!(fn_with_python_stub, m)?)?;
    m.add_function(wrap_pyfunction!(overload_example_1, m)?)?;
    m.add_function(wrap_pyfunction!(overload_example_2, m)?)?;
    m.add_function(wrap_pyfunction!(as_tuple, m)?)?;
    m.add_function(wrap_pyfunction!(add_decimals, m)?)?;
    m.add_function(wrap_pyfunction!(process_container, m)?)?;
    m.add_function(wrap_pyfunction!(sum_list, m)?)?;
    m.add_function(wrap_pyfunction!(create_containers, m)?)?;
    // Test-cases for `*args` and `**kwargs`
    m.add_function(wrap_pyfunction!(func_with_star_arg, m)?)?;
    m.add_function(wrap_pyfunction!(func_with_star_arg_typed, m)?)?;
    m.add_function(wrap_pyfunction!(func_with_kwargs, m)?)?;

    // Test cases for type: ignore functionality
    m.add_function(wrap_pyfunction!(test_type_ignore_specific, m)?)?;
    m.add_function(wrap_pyfunction!(test_type_ignore_all, m)?)?;
    m.add_function(wrap_pyfunction!(test_type_ignore_pyright, m)?)?;
    m.add_function(wrap_pyfunction!(test_type_ignore_custom, m)?)?;
    m.add_function(wrap_pyfunction!(test_type_ignore_no_comment_all, m)?)?;
    m.add_function(wrap_pyfunction!(test_type_ignore_no_comment_specific, m)?)?;

    // Test case for custom exceptions
    m.add("MyError", m.py().get_type::<MyError>())?;
    m.add_class::<NotIntError>()?;

    // Test class for type: ignore functionality
    m.add_class::<TypeIgnoreTest>()?;
    Ok(())
}

/// Test function with type: ignore for specific rules
#[gen_stub_pyfunction]
#[gen_stub(type_ignore = ["arg-type", "return-value"])]
#[pyfunction]
fn test_type_ignore_specific() -> i32 {
    42
}

/// Test function with type: ignore (without equals for catch-all)
#[gen_stub_pyfunction]
#[gen_stub(type_ignore)]
#[pyfunction]
fn test_type_ignore_all() -> i32 {
    42
}

/// Test function with Pyright diagnostic rules
#[gen_stub_pyfunction]
#[gen_stub(type_ignore = ["reportGeneralTypeIssues", "reportReturnType"])]
#[pyfunction]
fn test_type_ignore_pyright() -> i32 {
    42
}

/// Test function with custom (unknown) rule
#[gen_stub_pyfunction]
#[gen_stub(type_ignore = ["custom-rule", "attr-defined"])]
#[pyfunction]
fn test_type_ignore_custom() -> i32 {
    42
}

// NOTE: Doc-comment MUST NOT be added to the next function,
// as it tests if `type_ignore` without no doccomment is handled correctly;
// i.e. it emits comment after `...`, not before.

#[gen_stub_pyfunction]
#[gen_stub(type_ignore)]
#[pyfunction]
fn test_type_ignore_no_comment_all() -> i32 {
    42
}

#[gen_stub_pyfunction]
#[gen_stub(type_ignore=["arg-type", "reportIncompatibleMethodOverride"])]
#[pyfunction]
fn test_type_ignore_no_comment_specific() -> i32 {
    42
}

/// Test class for method type: ignore functionality
#[gen_stub_pyclass]
#[pyclass]
pub struct TypeIgnoreTest {}

#[gen_stub_pymethods]
#[pymethods]
impl TypeIgnoreTest {
    #[new]
    fn new() -> Self {
        Self {}
    }

    /// Test method with type: ignore for specific rules
    #[gen_stub(type_ignore = ["union-attr", "return-value"])]
    fn test_method_ignore(&self, value: i32) -> i32 {
        value * 2
    }

    /// Test method with type: ignore (without equals for catch-all)
    #[gen_stub(type_ignore)]
    fn test_method_all_ignore(&self) -> i32 {
        42
    }
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
