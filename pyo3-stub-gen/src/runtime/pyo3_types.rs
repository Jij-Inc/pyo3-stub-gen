//! PyRuntimeType implementations for PyO3 wrapper types.
//!
//! This module provides `PyRuntimeType` implementations for PyO3's wrapper types
//! (`Bound<T>`, `Py<T>`) that delegate to the wrapped type's `PyTypeInfo`.
//!
//! Unlike primitive types which map to Python built-ins, these implementations
//! use PyO3's type system to obtain the actual Python class object.

use super::PyRuntimeType;
use pyo3::prelude::*;
use pyo3::type_object::PyTypeInfo;
use pyo3::types::PyAny;

/// Implementation for `Bound<'_, T>` where T implements PyTypeInfo.
///
/// This allows type aliases to reference PyO3 classes:
/// ```rust,ignore
/// define_type_alias! {
///     pub struct MyUnion in "module" = Bound<'static, MyClass> | Bound<'static, OtherClass>;
/// }
/// ```
impl<T: PyTypeInfo> PyRuntimeType for Bound<'_, T> {
    fn py_type(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        Ok(T::type_object(py).into_any())
    }
}

/// Implementation for `Py<T>` where T implements PyTypeInfo.
///
/// This allows type aliases to use owned Python references:
/// ```rust,ignore
/// define_type_alias! {
///     pub struct MyUnion in "module" = Py<MyClass> | i32;
/// }
/// ```
impl<T: PyTypeInfo> PyRuntimeType for Py<T> {
    fn py_type(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        Ok(T::type_object(py).into_any())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::types::{PyDict, PyList, PyString};

    #[test]
    fn test_bound_wrapper() {
        pyo3::Python::initialize();
        Python::attach(|py| {
            // Bound<PyString> should return the str type
            let str_type = <Bound<'_, PyString> as PyRuntimeType>::py_type(py).unwrap();
            let expected = py.get_type::<PyString>().into_any();
            assert!(str_type.is(&expected));

            let list_type = <Bound<'_, PyList> as PyRuntimeType>::py_type(py).unwrap();
            let expected = py.get_type::<PyList>().into_any();
            assert!(list_type.is(&expected));

            let dict_type = <Bound<'_, PyDict> as PyRuntimeType>::py_type(py).unwrap();
            let expected = py.get_type::<PyDict>().into_any();
            assert!(dict_type.is(&expected));
        });
    }

    #[test]
    fn test_py_wrapper() {
        pyo3::Python::initialize();
        Python::attach(|py| {
            // Py<PyString> should return the str type
            let str_type = <Py<PyString> as PyRuntimeType>::py_type(py).unwrap();
            let expected = py.get_type::<PyString>().into_any();
            assert!(str_type.is(&expected));
        });
    }
}
