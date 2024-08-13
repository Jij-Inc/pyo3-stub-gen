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
