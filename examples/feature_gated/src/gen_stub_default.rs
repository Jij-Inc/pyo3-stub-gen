use pyo3::prelude::*;

// To use `gen_stub` annotations with feature-gating, you need to remove them when the feature is off.
// It can be done by `remove_gen_stub` attribute macro.
#[cfg_attr(feature = "stub-gen", pyo3_stub_gen_derive::gen_stub_pyclass)]
#[cfg_attr(not(feature = "stub-gen"), pyo3_stub_gen_derive::remove_gen_stub)]
#[pyclass]
#[derive(Default)]
pub struct A {
    #[pyo3(get, set)]
    // You cannot use `cfg_attr` here like following since `gen_stub` is not attribute macro.
    // #[cfg_attr(feature = "stub-gen", gen_stub(default = A::default().x))]
    #[gen_stub(default = A::default().x)]
    x: usize,
    y: usize,
}

#[cfg_attr(feature = "stub-gen", pyo3_stub_gen_derive::gen_stub_pymethods)]
#[cfg_attr(not(feature = "stub-gen"), pyo3_stub_gen_derive::remove_gen_stub)]
#[pymethods]
impl A {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    #[gen_stub(default = A::default().y)]
    pub fn get_y(&self) -> usize {
        self.y
    }
}
