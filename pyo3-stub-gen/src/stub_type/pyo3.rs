use crate::stub_type::*;
use ::pyo3::{
    basic::CompareOp,
    pybacked::{PyBackedBytes, PyBackedStr},
    pyclass::boolean_struct::False,
    types::*,
    Bound, Py, PyClass, PyRef, PyRefMut,
};
use maplit::hashset;

impl PyStubType for PyAny {
    fn type_output(_current_module_name: &str) -> TypeInfo {
        TypeInfo {
            name: "typing.Any".to_string(),
            import: hashset! { "typing".into() },
        }
    }
}

impl<T: PyStubType> PyStubType for Py<T> {
    fn type_input(current_module_name: &str) -> TypeInfo {
        T::type_input(current_module_name)
    }
    fn type_output(current_module_name: &str) -> TypeInfo {
        T::type_output(current_module_name)
    }
}

impl<T: PyStubType + PyClass> PyStubType for PyRef<'_, T> {
    fn type_input(current_module_name: &str) -> TypeInfo {
        T::type_input(current_module_name)
    }
    fn type_output(current_module_name: &str) -> TypeInfo {
        T::type_output(current_module_name)
    }
}

impl<T: PyStubType + PyClass<Frozen = False>> PyStubType for PyRefMut<'_, T> {
    fn type_input(current_module_name: &str) -> TypeInfo {
        T::type_input(current_module_name)
    }
    fn type_output(current_module_name: &str) -> TypeInfo {
        T::type_output(current_module_name)
    }
}

impl<T: PyStubType> PyStubType for Bound<'_, T> {
    fn type_input(current_module_name: &str) -> TypeInfo {
        T::type_input(current_module_name)
    }
    fn type_output(current_module_name: &str) -> TypeInfo {
        T::type_output(current_module_name)
    }
}

macro_rules! impl_builtin {
    ($ty:ty, $pytype:expr) => {
        impl PyStubType for $ty {
            fn type_output(_current_module_name: &str) -> TypeInfo {
                TypeInfo {
                    name: $pytype.to_string(),
                    import: HashSet::new(),
                }
            }
        }
    };
}

impl_builtin!(PyInt, "int");
impl_builtin!(PyFloat, "float");
impl_builtin!(PyList, "list");
impl_builtin!(PyTuple, "tuple");
impl_builtin!(PySlice, "slice");
impl_builtin!(PyDict, "dict");
impl_builtin!(PySet, "set");
impl_builtin!(PyString, "str");
impl_builtin!(PyBackedStr, "str");
impl_builtin!(PyByteArray, "bytearray");
impl_builtin!(PyBytes, "bytes");
impl_builtin!(PyBackedBytes, "bytes");
impl_builtin!(PyType, "type");
impl_builtin!(CompareOp, "int");

macro_rules! impl_simple {
    ($ty:ty, $mod:expr, $pytype:expr) => {
        impl PyStubType for $ty {
            fn type_output(_current_module_name: &str) -> TypeInfo {
                TypeInfo {
                    name: concat!($mod, ".", $pytype).to_string(),
                    import: hashset! { $mod.into() },
                }
            }
        }
    };
}

impl_simple!(PyDate, "datetime", "date");
impl_simple!(PyDateTime, "datetime", "datetime");
impl_simple!(PyDelta, "datetime", "timedelta");
impl_simple!(PyTime, "datetime", "time");
impl_simple!(PyTzInfo, "datetime", "tzinfo");
