use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

/// Test class with manually defined PyStubType
#[gen_stub_pyclass(skip_stub_type)]
#[pyclass]
pub struct CustomStubType {
    #[pyo3(get, set)]
    pub value: i32,
}

#[gen_stub_pymethods]
#[pymethods]
impl CustomStubType {
    #[new]
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    pub fn increment(&mut self) -> i32 {
        self.value += 1;
        self.value
    }
}

// Manually implement PyStubType with custom type representation
// This demonstrates that skip_stub_type allows us to provide a custom implementation
// Here we're using a type alias to show it's different from the auto-generated one
impl pyo3_stub_gen::PyStubType for CustomStubType {
    fn type_output() -> pyo3_stub_gen::TypeInfo {
        // You could customize this to use a different type name, module, etc.
        // For now, we keep it simple but this proves skip_stub_type works
        pyo3_stub_gen::TypeInfo::with_module("CustomStubType", "pure".into())
    }
}

/// Test class without skip_stub_type (normal behavior)
#[gen_stub_pyclass]
#[pyclass]
pub struct NormalClass {
    #[pyo3(get, set)]
    pub value: String,
}

#[gen_stub_pymethods]
#[pymethods]
impl NormalClass {
    #[new]
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

/// Test enum with skip_stub_type
#[gen_stub_pyclass_enum(skip_stub_type)]
#[pyclass]
pub enum CustomEnum {
    #[pyo3(name = "OPTION_A")]
    OptionA,
    #[pyo3(name = "OPTION_B")]
    OptionB,
}

// Manually implement PyStubType for the enum
impl pyo3_stub_gen::PyStubType for CustomEnum {
    fn type_output() -> pyo3_stub_gen::TypeInfo {
        pyo3_stub_gen::TypeInfo::with_module("CustomEnum", "pure".into())
    }
}

/// Test complex enum with skip_stub_type
#[gen_stub_pyclass_complex_enum(skip_stub_type)]
#[pyclass]
pub enum CustomComplexEnum {
    #[pyo3(name = "VARIANT_A")]
    VariantA { value: i32 },
    #[pyo3(name = "VARIANT_B")]
    VariantB(String),
}

// Manually implement PyStubType for the complex enum
impl pyo3_stub_gen::PyStubType for CustomComplexEnum {
    fn type_output() -> pyo3_stub_gen::TypeInfo {
        pyo3_stub_gen::TypeInfo::with_module("CustomComplexEnum", "pure".into())
    }
}
