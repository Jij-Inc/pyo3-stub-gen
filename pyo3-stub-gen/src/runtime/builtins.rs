//! PyRuntimeType implementations for Rust primitive types.

use super::PyRuntimeType;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyBytes, PyFloat, PyInt, PyString, PyType};

macro_rules! impl_runtime_builtin {
    ($rust_ty:ty, $py_ty:ty) => {
        impl PyRuntimeType for $rust_ty {
            fn py_type(py: Python<'_>) -> PyResult<Bound<'_, PyType>> {
                Ok(py.get_type::<$py_ty>())
            }
        }
    };
}

// Integer types -> int
impl_runtime_builtin!(i8, PyInt);
impl_runtime_builtin!(i16, PyInt);
impl_runtime_builtin!(i32, PyInt);
impl_runtime_builtin!(i64, PyInt);
impl_runtime_builtin!(i128, PyInt);
impl_runtime_builtin!(isize, PyInt);
impl_runtime_builtin!(u8, PyInt);
impl_runtime_builtin!(u16, PyInt);
impl_runtime_builtin!(u32, PyInt);
impl_runtime_builtin!(u64, PyInt);
impl_runtime_builtin!(u128, PyInt);
impl_runtime_builtin!(usize, PyInt);

// Float types -> float
impl_runtime_builtin!(f32, PyFloat);
impl_runtime_builtin!(f64, PyFloat);

// String types -> str
impl_runtime_builtin!(String, PyString);
impl_runtime_builtin!(&str, PyString);
impl_runtime_builtin!(char, PyString);
impl_runtime_builtin!(std::borrow::Cow<'_, str>, PyString);

// Bool -> bool
impl_runtime_builtin!(bool, PyBool);

// Bytes -> bytes
impl_runtime_builtin!(Vec<u8>, PyBytes);
impl_runtime_builtin!(&[u8], PyBytes);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_types() {
        pyo3::Python::initialize();
        Python::attach(|py| {
            let int_type = py.get_type::<PyInt>();

            assert!(i32::py_type(py).unwrap().is(&int_type));
            assert!(u64::py_type(py).unwrap().is(&int_type));
            assert!(isize::py_type(py).unwrap().is(&int_type));
        });
    }

    #[test]
    fn test_float_types() {
        pyo3::Python::initialize();
        Python::attach(|py| {
            let float_type = py.get_type::<PyFloat>();

            assert!(f32::py_type(py).unwrap().is(&float_type));
            assert!(f64::py_type(py).unwrap().is(&float_type));
        });
    }

    #[test]
    fn test_string_types() {
        pyo3::Python::initialize();
        Python::attach(|py| {
            let str_type = py.get_type::<PyString>();

            assert!(String::py_type(py).unwrap().is(&str_type));
            assert!(<&str>::py_type(py).unwrap().is(&str_type));
        });
    }

    #[test]
    fn test_bool_type() {
        pyo3::Python::initialize();
        Python::attach(|py| {
            let bool_type = py.get_type::<PyBool>();
            assert!(bool::py_type(py).unwrap().is(&bool_type));
        });
    }
}
