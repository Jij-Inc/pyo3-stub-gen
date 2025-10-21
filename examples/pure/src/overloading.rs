//! Test for `@overload` decorator generation

use pyo3::{exceptions::PyTypeError, prelude::*, types::PyTuple, IntoPyObjectExt, PyObject};
use pyo3_stub_gen::{derive::*, inventory::submit};

/// First example: One generated with ordinary `#[gen_stub_pyfunction]`,
/// and then manually with `submit!` macro.
#[gen_stub_pyfunction]
#[pyfunction]
pub fn overload_example_1(x: f64) -> f64 {
    x + 1.0
}

submit! {
    gen_function_from_python! {
        r#"
        def overload_example_1(x: int) -> int: ...
        "#
    }
}
/// Second example: all hints manually `submit!`ed via macro.
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

submit! {
    gen_function_from_python! {
        r#"
        def overload_example_2(ob: float) -> float:
            """Increments float by 1"""
        "#
    }
}

submit! {
    gen_function_from_python! {
        r#"
        def overload_example_2(ob: int) -> int:
            """Increments integer by 1"""
        "#
    }
}

/// Example using Literal[True] and Literal[False] for overloading.
/// This is a common pattern for functions that return different types based on a boolean flag.
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

submit! {
    gen_function_from_python! {
        r#"
        import typing
        import collections.abc
        def as_tuple(xs: collections.abc.Sequence[int], /, *, tuple_out: typing.Literal[True]) -> tuple[int, ...]:
            """Convert sequence to tuple when tuple_out is True"""
        "#
    }
}

submit! {
    gen_function_from_python! {
        r#"
        import typing
        import collections.abc
        def as_tuple(xs: collections.abc.Sequence[int], /, *, tuple_out: typing.Literal[False]) -> list[int]:
            """Convert sequence to list when tuple_out is False"""
        "#
    }
}
