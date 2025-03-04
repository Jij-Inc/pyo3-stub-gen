use super::{PyStubType, TypeInfo};
use maplit::hashset;
use pyo3_arrow::{PyRecordBatchReader, PySchema};

impl PyStubType for PyRecordBatchReader {
    fn type_output() -> TypeInfo {
        TypeInfo {
            name: "pyarrow.RecordBatchReader".into(),
            import: hashset!["pyarrow".into()],
        }
    }
}

impl PyStubType for PySchema {
    fn type_output() -> TypeInfo {
        TypeInfo {
            name: "pyarrow.Schema".into(),
            import: hashset!["pyarrow".into()],
        }
    }
}
