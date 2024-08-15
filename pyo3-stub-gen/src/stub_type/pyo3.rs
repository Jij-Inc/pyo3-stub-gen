use crate::stub_type::*;
use ::pyo3::types::*;
use maplit::hashset;

impl PyStubType for PyAny {
    fn type_output() -> TypeInfo {
        TypeInfo {
            name: "typing.Any".to_string(),
            import: hashset! { "typing".into() },
        }
    }
}

macro_rules! impl_builtin {
    ($ty:ty, $pytype:expr) => {
        impl PyStubType for $ty {
            fn type_output() -> TypeInfo {
                TypeInfo {
                    name: $pytype.to_string(),
                    import: HashSet::new(),
                }
            }
        }
    };
}

impl_builtin!(PyList, "list");
impl_builtin!(PyDict, "dict");
impl_builtin!(PyString, "str");
impl_builtin!(PyByteArray, "bytearray");
impl_builtin!(PyBytes, "bytes");
