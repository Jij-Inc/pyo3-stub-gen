use super::{PyStubType, TypeInfo};
use ::pyo3::prelude::*;

impl PyStubType for rust_decimal::Decimal {
    fn type_output() -> TypeInfo {
        TypeInfo::with_module("decimal.Decimal", "decimal".into())
    }
    fn type_object(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        let decimal = py.import("decimal")?;
        decimal.getattr("Decimal")
    }
}
