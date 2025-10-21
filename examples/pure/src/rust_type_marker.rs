//! Example of using pyo3_stub_gen.RustType["..."] marker in Python stub syntax
//!
//! This demonstrates Issue #328: allowing Python stubs to inline Rust-side type information
//! using the RustType marker.

use pyo3::prelude::*;
use pyo3_stub_gen::{derive::*, inventory::submit};

// Define a Rust type that will be exposed to Python
#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct DataContainer {
    #[pyo3(get, set)]
    pub value: i32,
}

#[gen_stub_pymethods]
#[pymethods]
impl DataContainer {
    #[new]
    fn new(value: i32) -> Self {
        Self { value }
    }
}

// Example 1: Function with RustType marker for both argument and return type
#[pyfunction]
pub fn process_container(container: Py<DataContainer>) -> PyResult<Py<DataContainer>> {
    Python::with_gil(|py| {
        let mut c = container.borrow_mut(py);
        c.value *= 2;
        drop(c);
        Ok(container)
    })
}

submit! {
    gen_function_from_python! {
        r#"
        def process_container(container: pyo3_stub_gen.RustType["DataContainer"]) -> pyo3_stub_gen.RustType["DataContainer"]:
            """
            Process a DataContainer by doubling its value.

            This uses the RustType marker to reference the Rust type directly,
            which will expand to the correct Python stub type using PyStubType trait.
            """
        "#
    }
}

// Example 2: Function with generic Rust types
#[pyfunction]
pub fn sum_list(values: Vec<i32>) -> i32 {
    values.iter().sum()
}

submit! {
    gen_function_from_python! {
        r#"
        def sum_list(values: pyo3_stub_gen.RustType["Vec<i32>"]) -> pyo3_stub_gen.RustType["i32"]:
            """
            Sum a list of integers.

            RustType["Vec<i32>"] will expand to the correct input type (typing.Sequence[int])
            and RustType["i32"] will expand to the correct output type (int).
            """
        "#
    }
}

// Example 3: Method with RustType marker
#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct Calculator {
    result: f64,
}

#[gen_stub_pymethods]
#[pymethods]
impl Calculator {
    #[new]
    fn new() -> Self {
        Self { result: 0.0 }
    }

    fn add(&mut self, value: f64) -> f64 {
        self.result += value;
        self.result
    }

    #[gen_stub(skip)]
    fn multiply(&mut self, other: Self) -> Self {
        self.result *= other.result;
        self.clone()
    }
}

submit! {
    gen_methods_from_python! {
        r#"
        class Calculator:
            def multiply(self, other: pyo3_stub_gen.RustType["Calculator"]) -> pyo3_stub_gen.RustType["Calculator"]:
                """
                Multiply this calculator's result by another calculator's result.

                Using RustType marker for both input and output types.
                """
        "#
    }
}

// Example 4: Complex type with module path
#[pyfunction]
pub fn create_containers(count: usize) -> Vec<DataContainer> {
    (0..count as i32)
        .map(|i| DataContainer { value: i })
        .collect()
}

submit! {
    gen_function_from_python! {
        r#"
        def create_containers(count: pyo3_stub_gen.RustType["usize"]) -> pyo3_stub_gen.RustType["Vec<DataContainer>"]:
            """
            Create a list of DataContainer instances.

            Demonstrates using RustType with generic types containing custom types.
            """
        "#
    }
}

// Example 5: RustType marker nested in complex type annotations
// This demonstrates using RustType markers within nested generic types like dict values

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct InstanceValue {
    #[pyo3(get, set)]
    pub data: String,
}

#[gen_stub_pymethods]
#[pymethods]
impl InstanceValue {
    #[new]
    fn new(data: String) -> Self {
        Self { data }
    }
}

#[gen_stub_pyclass]
#[pyclass]
pub struct Problem {}

#[gen_stub_pymethods]
#[pymethods]
impl Problem {
    #[new]
    fn new() -> Self {
        Self {}
    }

    #[gen_stub(skip)]
    fn evaluate(
        &self,
        instance_data: std::collections::HashMap<String, Py<InstanceValue>>,
    ) -> String {
        let _ = instance_data;
        "result".to_string()
    }
}

submit! {
    gen_methods_from_python! {
        r#"
        import builtins

        class Problem:
            def evaluate(
                self,
                instance_data: builtins.dict[
                    builtins.str,
                    pyo3_stub_gen.RustType["InstanceValue"]
                ]
            ) -> builtins.str:
                """
                Evaluate with instance data mapping string keys to InstanceValue objects.

                This example demonstrates RustType marker usage within nested generic types
                such as dict value types. The marker should expand to the correct Python type.
                """
        "#
    }
}

// Example 6: Keyword-only parameters (after *)
// This demonstrates Issue with keyword-only parameters being ignored
#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct Placeholder {
    #[pyo3(get, set)]
    pub name: String,
}

#[gen_stub_pymethods]
#[pymethods]
impl Placeholder {
    #[new]
    fn new(name: String) -> Self {
        Self { name }
    }

    #[gen_stub(skip)]
    #[pyo3(signature = (name, *, dtype, ndim, shape, jagged = false, latex = None))]
    fn configure(
        &self,
        name: String,
        dtype: String,
        ndim: i32,
        shape: Option<String>,
        jagged: bool,
        latex: Option<String>,
    ) -> Self {
        dbg!(&name, &dtype, &ndim, &shape, &jagged, &latex);
        self.clone()
    }
}

submit! {
    gen_methods_from_python! {
        r#"
        import builtins
        import typing

        class Placeholder:
            def configure(
                self,
                name: builtins.str,
                *,
                dtype: builtins.str,
                ndim: builtins.int,
                shape: typing.Optional[builtins.str],
                jagged: builtins.bool = False,
                latex: typing.Optional[builtins.str] = None,
            ) -> pyo3_stub_gen.RustType["Placeholder"]:
                """
                Configure placeholder with keyword-only parameters.

                This demonstrates keyword-only parameters (after *) which should be
                preserved in the generated stub file.
                """
        "#
    }
}
