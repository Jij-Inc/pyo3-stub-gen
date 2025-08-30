use pyo3::prelude::*;

#[cfg_attr(feature = "stub-gen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
pub struct A {
    #[pyo3(get, set)]
    #[cfg_attr(feature = "stub-gen", gen_stub(default = A::default().x))]
    x: usize,
    y: usize,
}

impl Default for A {
    fn default() -> Self {
        A { x: 0, y: 0 }
    }
}

#[cfg_attr(feature = "stub-gen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl A {
    #[cfg_attr(feature = "stub-gen", gen_stub(default = A::default().y))]
    pub fn get_y(&self) -> usize {
        self.y
    }
}
