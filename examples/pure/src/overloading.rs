//! Test for `@overload` decorator generation

use pyo3::{exceptions::PyTypeError, prelude::*, IntoPyObjectExt, PyObject};
use pyo3_stub_gen::{
    derive::*,
    inventory::submit,
    type_info::{ArgInfo, PyFunctionInfo},
    PyStubType,
};

/// First example: One generated with ordinary `#[gen_stub_pyfunction]`,
/// and then manually with `submit!` macro.
#[gen_stub_pyfunction]
#[pyfunction]
pub fn overload_example_1(x: f64) -> f64 {
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
        type_ignored: None,
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
    PyFunctionInfo {
        name: "overload_example_2",
        args: &[ArgInfo{
            name: "ob",
            signature: None,
            r#type: || f64::type_input(),
        }],
        r#return: || f64::type_output(),
        module: None,
        doc: "Increments float by 1",
        is_async: false,
        deprecated: None,
        type_ignored: None,
    }
}

submit! {
    PyFunctionInfo {
        name: "overload_example_2",
        args: &[ArgInfo{
            name: "ob",
            signature: None,
            r#type: || i64::type_input(),
        }],
        r#return: || i64::type_output(),
        module: None,
        doc: "Increments integer by 1",
        is_async: false,
        deprecated: None,
        type_ignored: None,
    }
}
