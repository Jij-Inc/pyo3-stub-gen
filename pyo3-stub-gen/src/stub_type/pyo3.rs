use crate::stub_type::*;
use ::pyo3::types::*;
use maplit::hashset;

impl PyStubType for PyAny {
    fn type_output() -> TypeInfo {
        TypeInfo {
            name: "typing.Any".to_string(),
            import: hashset! { "typing".to_string() },
        }
    }
}

impl PyStubType for PyList {
    fn type_output() -> TypeInfo {
        TypeInfo {
            name: "list".to_string(),
            import: HashSet::new(),
        }
    }
}

impl PyStubType for PyDict {
    fn type_output() -> TypeInfo {
        TypeInfo {
            name: "dict".to_string(),
            import: HashSet::new(),
        }
    }
}
