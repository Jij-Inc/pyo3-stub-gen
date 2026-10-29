//! Share external Markdown and macro-generated documentation with PyO3 and stubs.

use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass]
#[pyclass(module = "pure")]
/// Inline comment.
#[doc = " Literal attribute."]
#[doc = include_str!("docs/external.md")]
#[doc = concat!("Concatenated ", "text.")]
/// Final comment.
pub struct DocumentedClass {
    #[pyo3(get, set)]
    #[doc = include_str!("docs/external.md")]
    value: u32,
}

#[gen_stub_pymethods]
#[pymethods]
impl DocumentedClass {
    #[new]
    fn new(value: u32) -> Self {
        Self { value }
    }

    #[doc = include_str!("docs/external.md")]
    fn answer(&self) -> u32 {
        self.value
    }

    #[getter]
    #[doc = concat!("Getter ", "docs.")]
    fn computed(&self) -> u32 {
        self.value
    }

    #[setter]
    #[doc = concat!("Setter ", "docs.")]
    fn set_computed(&mut self, value: u32) {
        self.value = value;
    }

    #[classattr]
    #[doc = concat!("Constant ", "docs.")]
    const LIMIT: u32 = 42;

    #[classattr]
    #[doc = include_str!("docs/external.md")]
    fn description() -> &'static str {
        "description"
    }
}

#[gen_stub_pyfunction]
#[pyfunction]
#[doc = concat!("Free function ", "docs.")]
#[doc = ""]
#[doc = include_str!("docs/external.md")]
#[doc = concat!("    assert_eq!(6 * 7, ", "42);")]
#[doc = concat!("Last ", "macro.")]
pub fn documented_answer() -> u32 {
    42
}

#[gen_stub_pyclass_complex_enum]
#[pyclass(module = "pure", skip_from_py_object)]
#[derive(Clone)]
#[doc = include_str!("docs/external.md")]
pub enum DocumentedComplexEnum {
    #[doc = concat!("Structured variant ", "docs.")]
    Value {
        #[doc = include_str!("docs/external.md")]
        value: u32,
    },
}
