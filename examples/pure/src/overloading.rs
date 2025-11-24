//! Test for `@overload` decorator generation

use pyo3::{exceptions::PyTypeError, prelude::*, types::PyTuple, IntoPyObjectExt, PyObject};
use pyo3_stub_gen::derive::*;

// Example 1: Using new python_overload parameter
// This adds an int overload while also generating the float overload from Rust types
#[gen_stub_pyfunction(python_overload = r#"
    @overload
    def overload_example_1(x: int) -> int: ...
    "#)]
#[pyfunction]
pub fn overload_example_1(x: f64) -> f64 {
    x + 1.0
}

// Example 2: Using python_overload with no_default_overload
// This suppresses auto-generation from Rust types (Bound<PyAny> is not useful for typing)
// Note: More specific type (int) should come first for Python overload rules
#[gen_stub_pyfunction(
    python_overload = r#"
    @overload
    def overload_example_2(ob: int) -> int:
        """Increments integer by 1"""

    @overload
    def overload_example_2(ob: float) -> float:
        """Increments float by 1"""
    "#,
    no_default_overload = true
)]
#[pyfunction]
pub fn overload_example_2(ob: Bound<PyAny>) -> PyResult<PyObject> {
    let py = ob.py();
    if let Ok(f) = ob.extract::<f64>() {
        (f + 1.0).into_py_any(py)
    } else if let Ok(i) = ob.extract::<i64>() {
        (i + 1).into_py_any(py)
    } else {
        Err(PyTypeError::new_err("Invalid type, expected float or int"))
    }
}

// Example 3: Using Literal[True] and Literal[False] for overloading
// This is a common pattern for functions that return different types based on a boolean flag
#[gen_stub_pyfunction(
    python_overload = r#"
    import typing
    import collections.abc

    @overload
    def as_tuple(xs: collections.abc.Sequence[int], /, *, tuple_out: typing.Literal[True]) -> tuple[int, ...]:
        """Convert sequence to tuple when tuple_out is True"""

    @overload
    def as_tuple(xs: collections.abc.Sequence[int], /, *, tuple_out: typing.Literal[False]) -> list[int]:
        """Convert sequence to list when tuple_out is False"""
    "#,
    no_default_overload = true
)]
#[pyfunction]
#[pyo3(signature = (xs, /, *, tuple_out))]
pub fn as_tuple(xs: Vec<i32>, tuple_out: bool) -> PyResult<PyObject> {
    Python::with_gil(|py| {
        if tuple_out {
            Ok(PyTuple::new(py, xs.iter())?.into_py_any(py)?)
        } else {
            Ok(xs.into_py_any(py)?)
        }
    })
}
