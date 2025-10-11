use pyo3::prelude::*;
use pyo3_stub_gen::{derive::*, inventory::submit};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[pyclass]
#[gen_stub_pyclass]
pub struct Incrementer {}

#[gen_stub_pymethods]
#[pymethods]
impl Incrementer {
    #[new]
    fn new() -> Self {
        Incrementer {}
    }

    /// This is the original doc comment
    fn increment_1(&self, x: f64) -> f64 {
        x + 1.0
    }
}

submit! {
    gen_methods_from_python! {
        r#"
        class Incrementer:
            def increment_1(self, x: int) -> int:
                """And this is for the second comment"""
        "#
    }
}

// Next, without gen_stub_pymethods and all submitted manually
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[pyclass]
#[gen_stub_pyclass]
pub struct Incrementer2 {}

#[pymethods]
impl Incrementer2 {
    #[new]
    fn new() -> Self {
        Incrementer2 {}
    }

    fn increment_2(&self, x: f64) -> f64 {
        x + 2.0
    }
}

submit! {
    gen_methods_from_python! {
        r#"
        class Incrementer2:
            def increment_2(self, x: int) -> int:
                """increment_2 for integers, submitted by hands"""

            def __new__(cls) -> Incrementer2:
                """Constructor for Incrementer2"""

            def increment_2(self, x: float) -> float:
                """increment_2 for floats, submitted by hands"""
        "#
    }
}
