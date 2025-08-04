#![allow(deprecated)]

#[cfg_attr(target_os = "macos", doc = include_str!("../../../README.md"))]
mod readme {}

use ahash::RandomState;
use pyo3::{
    exceptions::{PyRuntimeError, PyTypeError},
    prelude::*,
    types::*,
    IntoPyObjectExt,
};
use pyo3_stub_gen::{
    create_exception, define_stub_info_gatherer,
    derive::*,
    generate::MethodType,
    inventory::submit,
    module_variable,
    type_info::{ArgInfo, MethodInfo, PyFunctionInfo, PyMethodsInfo},
    PyStubType,
};
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
    
    /// deprecated class attribute NUM3
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
    fn deprecated_setter(&mut self, value: usize) {
        self.x = value;
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

module_variable!("pure", "MY_CONSTANT", usize);

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

#[gen_stub_pyfunction]
#[pyfunction]
#[gen_stub(override_return_type(type_repr="collections.abc.Callable[[str]]", imports=("collections.abc")))]
fn fn_override_type<'a>(
    #[gen_stub(override_type(type_repr="collections.abc.Callable[[str]]", imports=("collections.abc")))]
    cb: Bound<'a, PyAny>,
) -> PyResult<Bound<'a, PyAny>> {
    cb.call1(("Hello!",))?;
    Ok(cb)
}
#[gen_stub_pyclass]
#[pyclass]
struct OverrideType {
    num: isize,
}

#[gen_stub_pymethods]
#[pymethods]
impl OverrideType {
    #[gen_stub(override_return_type(type_repr="typing_extensions.Never", imports=("typing_extensions")))]
    fn error(&self) -> PyResult<()> {
        Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "I'm an error!",
        ))
    }

    #[getter]
    #[gen_stub(override_return_type(type_repr = "int"))]
    fn get_num(&self) -> PyResult<Py<PyAny>> {
        Python::with_gil(|py| self.num.into_py_any(py))
    }

    #[setter]
    fn set_num(
        &mut self,
        #[gen_stub(override_type(type_repr = "str"))] value: Py<PyAny>,
    ) -> PyResult<()> {
        self.num = Python::with_gil(|py| value.extract::<String>(py))?.parse::<isize>()?;
        Ok(())
    }
}

// Test for `@overload` decorator generation

/// First example: One generated with ordinary `#[gen_stub_pyfunction]`,
/// and then manually with `submit!` macro.
#[gen_stub_pyfunction]
#[pyfunction]
fn overload_example_1(x: f64) -> f64 {
    x + 1.0
}

submit! {
    PyFunctionInfo {
        name: "overload_example_1",
        args: &[ArgInfo{
            name: "x",
            signature: None,
            r#type: || i64::type_input(),
        }],
        r#return: || i64::type_output(),
        module: None,
        doc: "",
        is_async: false,
        deprecated: None,
    }
}
/// Second example: all hints manually `submit!`ed via macro.
#[pyfunction]
fn overload_example_2(ob: Bound<PyAny>) -> PyResult<PyObject> {
    let py = ob.py();
    if let Ok(f) = ob.extract::<f64>() {
        (f + 1.0).into_py_any(py)
    } else if let Ok(i) = ob.extract::<i64>() {
        (i + 1).into_py_any(py)
    } else {
        Err(PyTypeError::new_err("Invalid type, expected float or int"))
    }
}

submit! {
    PyFunctionInfo {
        name: "overload_example_2",
        args: &[ArgInfo{
            name: "x",
            signature: None,
            r#type: || f64::type_input(),
        }],
        r#return: || f64::type_output(),
        module: None,
        doc: "Increments float by 1",
        is_async: false,
        deprecated: None,
    }
}

submit! {
    PyFunctionInfo {
        name: "overload_example_2",
        args: &[ArgInfo{
            name: "x",
            signature: None,
            r#type: || i64::type_input(),
        }],
        r#return: || i64::type_output(),
        module: None,
        doc: "Increments integer by 1",
        is_async: false,
        deprecated: None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[pyclass]
#[gen_stub_pyclass]
pub struct Incrementer {}

#[pymethods]
#[gen_stub_pymethods]
impl Incrementer {
    #[new]
    fn new() -> Self {
        Incrementer {}
    }

    /// This is the original doc comment
    fn increment_1(&self, x: f64) -> f64 {
        x + 1.0
    }
}

submit! {
    PyMethodsInfo {
        struct_id: std::any::TypeId::of::<Incrementer>,
        attrs: &[],
        getters: &[],
        setters: &[],
        methods: &[
            MethodInfo {
                name: "increment_1",
                args: &[
                    ArgInfo {
                        name: "x",
                        signature: None,
                        r#type: || i64::type_input(),
                    },
                ],
                r#type: MethodType::Instance,
                r#return: || i64::type_output(),
                doc: "And this is for the second comment",
                is_async: false,
                deprecated: None,
            }
        ],
    }
}

// Next, without gen_stub_pymethods and all submitted manually
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[pyclass]
#[gen_stub_pyclass]
pub struct Incrementer2 {}

#[pymethods]
impl Incrementer2 {
    #[new]
    fn new() -> Self {
        Incrementer2 {}
    }

    fn increment_2(&self, x: f64) -> f64 {
        x + 2.0
    }
}

submit! {
    PyMethodsInfo {
        struct_id: std::any::TypeId::of::<Incrementer2>,
        attrs: &[],
        getters: &[],
        setters: &[],
        methods: &[
            MethodInfo {
                name: "increment_2",
                args: &[
                    ArgInfo {
                        name: "x",
                        signature: None,
                        r#type: || i64::type_input(),
                    },
                ],
                r#type: MethodType::Instance,
                r#return: || i64::type_output(),
                doc: "increment_2 for integers, submitted by hands",
                is_async: false,
                deprecated: None,
            },
            MethodInfo {
                name: "__new__",
                args: &[],
                r#type: MethodType::New,
                r#return: || Incrementer2::type_output(),
                doc: "Constructor for Incrementer2",
                is_async: false,
                deprecated: None,
            },
            MethodInfo {
                name: "increment_2",
                args: &[
                    ArgInfo {
                        name: "x",
                        signature: None,
                        r#type: || f64::type_input(),
                    },
                ],
                r#type: MethodType::Instance,
                r#return: || f64::type_output(),
                doc: "increment_2 for floats, submitted by hands",
                is_async: false,
                deprecated: None,
            },
        ],
    }
}

/// Initializes the Python module
#[pymodule]
fn pure(m: &Bound<PyModule>) -> PyResult<()> {
    m.add("MyError", m.py().get_type::<MyError>())?;
    m.add("MY_CONSTANT", 19937)?;
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
    m.add_function(wrap_pyfunction!(overload_example_1, m)?)?;
    m.add_function(wrap_pyfunction!(overload_example_2, m)?)?;
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
