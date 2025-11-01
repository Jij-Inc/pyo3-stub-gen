//! Test for `@overload` decorator generation using submit! syntax
//! This demonstrates the backward-compatible approach using submit! + gen_function_from_python!
//! and gen_methods_from_python! for class methods

use pyo3::{exceptions::PyTypeError, prelude::*, types::PyTuple, IntoPyObjectExt, PyObject};
use pyo3_stub_gen::{derive::*, inventory::submit};

// First example: One manually submitted via `submit!` macro, followed by one generated with `#[gen_stub_pyfunction]`.
// With the new implementation, @overload decorator is automatically detected and applied.

submit! {
    gen_function_from_python! {
        r#"
        @overload
        def manual_overload_example_1(x: int) -> int: ...
        "#
    }
}

#[gen_stub_pyfunction]
#[pyfunction]
pub fn manual_overload_example_1(x: f64) -> f64 {
    x + 1.0
}

/// Second example: all hints manually `submit!`ed via macro.
/// Note: More specific type (int) should come first for Python overload rules
#[pyfunction]
pub fn manual_overload_example_2(ob: Bound<PyAny>) -> PyResult<PyObject> {
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
    gen_function_from_python! {
        r#"
        @overload
        def manual_overload_example_2(ob: int) -> int:
            """Increments integer by 1"""
        "#
    }
}

submit! {
    gen_function_from_python! {
        r#"
        @overload
        def manual_overload_example_2(ob: float) -> float:
            """Increments float by 1"""
        "#
    }
}

/// Example using Literal[True] and Literal[False] for overloading.
/// This is a common pattern for functions that return different types based on a boolean flag.
#[pyfunction]
#[pyo3(signature = (xs, /, *, tuple_out))]
pub fn manual_overload_as_tuple(xs: Vec<i32>, tuple_out: bool) -> PyResult<PyObject> {
    Python::with_gil(|py| {
        if tuple_out {
            Ok(PyTuple::new(py, xs.iter())?.into_py_any(py)?)
        } else {
            Ok(xs.into_py_any(py)?)
        }
    })
}

submit! {
    gen_function_from_python! {
        r#"
        import typing
        import collections.abc
        @overload
        def manual_overload_as_tuple(xs: collections.abc.Sequence[int], /, *, tuple_out: typing.Literal[True]) -> tuple[int, ...]:
            """Convert sequence to tuple when tuple_out is True"""
        "#
    }
}

submit! {
    gen_function_from_python! {
        r#"
        import typing
        import collections.abc
        @overload
        def manual_overload_as_tuple(xs: collections.abc.Sequence[int], /, *, tuple_out: typing.Literal[False]) -> list[int]:
            """Convert sequence to list when tuple_out is False"""
        "#
    }
}

// ============================================================================
// Class Method Overloading Examples
// ============================================================================

/// Example 1: Class with overloaded instance method
/// Both signatures defined in Python stub to demonstrate method overloading
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[pyclass]
#[gen_stub_pyclass]
pub struct Incrementer {}

submit! {
    gen_methods_from_python! {
        r#"
        class Incrementer:
            def __new__(cls) -> Incrementer:
                """Constructor for Incrementer"""

            @overload
            def increment_1(self, x: int) -> int:
                """And this is for the second comment"""

            @overload
            def increment_1(self, x: float) -> float:
                """This is the original doc comment"""
        "#
    }
}

#[pymethods]
impl Incrementer {
    #[new]
    fn new() -> Self {
        Incrementer {}
    }

    fn increment_1(&self, x: f64) -> f64 {
        x + 1.0
    }
}

