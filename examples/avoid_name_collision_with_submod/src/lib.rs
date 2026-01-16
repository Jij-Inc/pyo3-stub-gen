use pyo3::prelude::*;
use pyo3_stub_gen::{derive::*, *};

#[gen_stub_pyclass_enum]
#[pyclass]
#[pyo3(eq, module = "avoid_name_collision_with_submod.sub_mod")]
#[derive(Clone, PartialEq, Eq)]
pub enum ClassA {
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
    pub fn ClassA(&self) -> ClassA {
        ClassA::Option1
    }

    pub fn collision(&self, a: ClassA) -> ClassA {
        a
    }

    #[pyo3(signature = (a = ClassA::Option1))]
    pub fn collision_with_def(&self, a: ClassA) -> ClassA {
        a
    }

    pub fn test_optional(&self, a: Option<ClassA>) -> Option<ClassA> {
        a
    }

    pub fn with_callback(&self, callback: ClassACallback) {
        let _ = callback;
    }
}

#[pyclass]
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
                    module: "avoid_name_collision_with_submod.submod".into(),
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
    sub.add_class::<ClassA>()?;
    m.add_submodule(&sub)?;
    Ok(())
}

define_stub_info_gatherer!(stub_info);
