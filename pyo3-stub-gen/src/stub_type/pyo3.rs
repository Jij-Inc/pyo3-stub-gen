use crate::stub_type::*;
use ::pyo3::{
    basic::CompareOp,
    pybacked::{PyBackedBytes, PyBackedStr},
    pyclass::boolean_struct::False,
    types::*,
    Bound, Py, PyClass, PyRef, PyRefMut, PyResult, Python,
};
use maplit::hashset;
use std::collections::HashMap;

impl PyStubType for PyAny {
    fn type_output() -> TypeInfo {
        TypeInfo {
            name: "typing.Any".to_string(),
            source_module: None,
            import: hashset! { "typing".into() },
            type_refs: HashMap::new(),
        }
    }
    fn type_object(py: Python<'_>) -> PyResult<Bound<'_, ::pyo3::PyAny>> {
        // PyAny maps to `object` at runtime
        Ok(py.get_type::<::pyo3::types::PyAny>().into_any())
    }
}

impl<T: PyStubType> PyStubType for Py<T> {
    fn type_input() -> TypeInfo {
        T::type_input()
    }
    fn type_output() -> TypeInfo {
        T::type_output()
    }
    fn type_object(py: Python<'_>) -> PyResult<Bound<'_, ::pyo3::PyAny>> {
        T::type_object(py)
    }
}

impl<T: PyStubType + PyClass> PyStubType for PyRef<'_, T> {
    fn type_input() -> TypeInfo {
        T::type_input()
    }
    fn type_output() -> TypeInfo {
        T::type_output()
    }
    fn type_object(py: Python<'_>) -> PyResult<Bound<'_, ::pyo3::PyAny>> {
        <T as PyStubType>::type_object(py)
    }
}

impl<T: PyStubType + PyClass<Frozen = False>> PyStubType for PyRefMut<'_, T> {
    fn type_input() -> TypeInfo {
        T::type_input()
    }
    fn type_output() -> TypeInfo {
        T::type_output()
    }
    fn type_object(py: Python<'_>) -> PyResult<Bound<'_, ::pyo3::PyAny>> {
        <T as PyStubType>::type_object(py)
    }
}

impl<T: PyStubType> PyStubType for Bound<'_, T> {
    fn type_input() -> TypeInfo {
        T::type_input()
    }
    fn type_output() -> TypeInfo {
        T::type_output()
    }
    fn type_object(py: Python<'_>) -> PyResult<Bound<'_, ::pyo3::PyAny>> {
        T::type_object(py)
    }
}

macro_rules! impl_builtin {
    ($ty:ty, $pytype:expr) => {
        impl PyStubType for $ty {
            fn type_output() -> TypeInfo {
                TypeInfo {
                    name: $pytype.to_string(),
                    source_module: None,
                    import: HashSet::new(),
                    type_refs: HashMap::new(),
                }
            }
            fn type_object(py: Python<'_>) -> PyResult<Bound<'_, ::pyo3::PyAny>> {
                Ok(py.get_type::<$ty>().into_any())
            }
        }
    };
}

impl_builtin!(PyBool, "bool");
impl_builtin!(PyInt, "int");
impl_builtin!(PyFloat, "float");
impl_builtin!(PyComplex, "complex");
impl_builtin!(PyList, "list");
impl_builtin!(PyTuple, "tuple");
impl_builtin!(PySlice, "slice");
impl_builtin!(PyDict, "dict");
impl_builtin!(PySet, "set");
impl_builtin!(PyString, "str");
impl_builtin!(PyByteArray, "bytearray");
impl_builtin!(PyBytes, "bytes");
impl_builtin!(PyType, "type");

// PyBackedStr and PyBackedBytes don't have PyTypeInfo, use underlying types
impl PyStubType for PyBackedStr {
    fn type_output() -> TypeInfo {
        TypeInfo {
            name: "str".to_string(),
            source_module: None,
            import: HashSet::new(),
            type_refs: HashMap::new(),
        }
    }
    fn type_object(py: Python<'_>) -> PyResult<Bound<'_, ::pyo3::PyAny>> {
        Ok(py.get_type::<PyString>().into_any())
    }
}

impl PyStubType for PyBackedBytes {
    fn type_output() -> TypeInfo {
        TypeInfo {
            name: "bytes".to_string(),
            source_module: None,
            import: HashSet::new(),
            type_refs: HashMap::new(),
        }
    }
    fn type_object(py: Python<'_>) -> PyResult<Bound<'_, ::pyo3::PyAny>> {
        Ok(py.get_type::<PyBytes>().into_any())
    }
}

// CompareOp maps to int at stub level but is not a Python type
impl PyStubType for CompareOp {
    fn type_output() -> TypeInfo {
        TypeInfo {
            name: "int".to_string(),
            source_module: None,
            import: HashSet::new(),
            type_refs: HashMap::new(),
        }
    }
    fn type_object(py: Python<'_>) -> PyResult<Bound<'_, ::pyo3::PyAny>> {
        Ok(py.get_type::<PyInt>().into_any())
    }
}

macro_rules! impl_simple {
    ($ty:ty, $mod:expr, $pytype:expr) => {
        impl PyStubType for $ty {
            fn type_output() -> TypeInfo {
                TypeInfo {
                    name: concat!($mod, ".", $pytype).to_string(),
                    source_module: None,
                    import: hashset! { $mod.into() },
                    type_refs: HashMap::new(),
                }
            }
            fn type_object(py: Python<'_>) -> PyResult<Bound<'_, ::pyo3::PyAny>> {
                Ok(py.get_type::<$ty>().into_any())
            }
        }
    };
}

impl_simple!(PyDate, "datetime", "date");
impl_simple!(PyDateTime, "datetime", "datetime");
impl_simple!(PyDelta, "datetime", "timedelta");
impl_simple!(PyTime, "datetime", "time");
impl_simple!(PyTzInfo, "datetime", "tzinfo");
