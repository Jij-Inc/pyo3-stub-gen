//! Test for `@overload` decorator generation

use pyo3::{exceptions::PyTypeError, prelude::*, IntoPyObjectExt, PyObject};
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
