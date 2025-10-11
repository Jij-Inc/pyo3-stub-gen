use pyo3::prelude::*;
use pyo3_stub_gen::{
    derive::*,
    generate::MethodType,
    inventory::submit,
    type_info::{ArgInfo, MethodInfo, PyMethodsInfo},
    PyStubType,
};

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
    PyMethodsInfo {
        struct_id: std::any::TypeId::of::<Incrementer>,
        attrs: &[],
        getters: &[],
        setters: &[],
        methods: &[
            MethodInfo {
                name: "increment_1",
                args: &[
                    ArgInfo {
                        name: "x",
                        signature: None,
                        r#type: || i64::type_input(),
                    },
                ],
                r#type: MethodType::Instance,
                r#return: || i64::type_output(),
                doc: "And this is for the second comment",
                is_async: false,
                deprecated: None,
                type_ignored: None,
            }
        ],
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
    PyMethodsInfo {
        struct_id: std::any::TypeId::of::<Incrementer2>,
        attrs: &[],
        getters: &[],
        setters: &[],
        methods: &[
            MethodInfo {
                name: "increment_2",
                args: &[
                    ArgInfo {
                        name: "x",
                        signature: None,
                        r#type: || i64::type_input(),
                    },
                ],
                r#type: MethodType::Instance,
                r#return: || i64::type_output(),
                doc: "increment_2 for integers, submitted by hands",
                is_async: false,
                deprecated: None,
                type_ignored: None,
            },
            MethodInfo {
                name: "__new__",
                args: &[],
                r#type: MethodType::New,
                r#return: || Incrementer2::type_output(),
                doc: "Constructor for Incrementer2",
                is_async: false,
                deprecated: None,
                type_ignored: None,
            },
            MethodInfo {
                name: "increment_2",
                args: &[
                    ArgInfo {
                        name: "x",
                        signature: None,
                        r#type: || f64::type_input(),
                    },
                ],
                r#type: MethodType::Instance,
                r#return: || f64::type_output(),
                doc: "increment_2 for floats, submitted by hands",
                is_async: false,
                deprecated: None,
                type_ignored: None,
            },
        ],
    }
}
