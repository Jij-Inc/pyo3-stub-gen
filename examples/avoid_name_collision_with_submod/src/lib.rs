use pyo3::prelude::*;
use pyo3_stub_gen::{derive::*, *};

#[gen_stub_pyclass_enum]
#[pyclass]
#[pyo3(
    eq,
    module = "avoid_name_collision_with_submod.sub_mod",
    name = "ClassA"
)]
#[derive(Clone, PartialEq, Eq)]
pub enum PyClassA {
    Option1,
    Option2,
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
#[pyo3(module = "avoid_name_collision_with_submod")]
pub struct ClassB {}

#[gen_stub_pymethods]
#[pymethods]
impl ClassB {
    #[allow(non_snake_case)]
    pub fn ClassA(&self) -> PyClassA {
        PyClassA::Option1
    }

    pub fn collision(&self, a: PyClassA) -> PyClassA {
        a
    }

    #[pyo3(signature = (a = PyClassA::Option1))]
    pub fn collision_with_def(&self, a: PyClassA) -> PyClassA {
        a
    }

    pub fn test_optional(&self, a: Option<PyClassA>) -> Option<PyClassA> {
        a
    }

    pub fn with_callback(&self, callback: ClassACallback) {
        let _ = callback;
    }

    pub fn classes_b(&self) -> Vec<PyClassA> {
        vec![]
    }
}

inventory::submit! {
    derive::gen_methods_from_python! {
        r#"
        class PyClassA:
            @property
            def my_elements(self) -> pyo3_stub_gen.RustType["Vec<PyClassA>"]:
                pass
        "#
    }
}

inventory::submit! {
    derive::gen_methods_from_python! {
        r#"
        class ClassB:
            @property
            def classes_b_manual(self, other: typing.Generator[pyo3_stub_gen.RustType["PyClassA"], None, None]) -> pyo3_stub_gen.RustType["Vec<PyClassA>"]:
                ...

            def who_am_i(self, other: pyo3_stub_gen.RustType["Vec<ClassB>"]) -> pyo3_stub_gen.RustType["Vec<ClassB>"]:
                ...
        "#
    }
}

#[pyclass]
#[allow(dead_code)]
pub struct ClassACallback(Py<PyAny>);

impl FromPyObject<'_> for ClassACallback {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        Ok(ClassACallback(ob.clone().unbind()))
    }
}

impl PyStubType for ClassACallback {
    fn type_output() -> TypeInfo {
        TypeInfo {
            name: "typing.Callable[[ClassA], ClassA]".to_string(),
            source_module: None,
            import: [].into(),
            type_refs: [(
                "ClassA".to_string(),
                TypeIdentifierRef {
                    // ClassA is the Python name for PyClassA (Rust enum) in sub_mod
                    module: "avoid_name_collision_with_submod.sub_mod".into(),
                    import_kind: ImportKind::Module,
                },
            )]
            .into(),
        }
    }
}

#[pymodule]
fn main_mod(m: &Bound<PyModule>) -> PyResult<()> {
    sub_mod(m)?;
    m.add_class::<ClassB>()?;
    Ok(())
}

fn sub_mod(m: &Bound<PyModule>) -> PyResult<()> {
    let py = m.py();
    let sub = PyModule::new(py, "sub_mod")?;
    sub.add_class::<PyClassA>()?;
    m.add_submodule(&sub)?;
    Ok(())
}

define_stub_info_gatherer!(stub_info);
