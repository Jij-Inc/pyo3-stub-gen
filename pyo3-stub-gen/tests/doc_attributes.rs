use pyo3::prelude::*;
use pyo3_stub_gen::{
    derive::*,
    docgen::{builder::DocPackageBuilder, ir::DocItem},
    inventory,
    type_info::*,
    StubInfo,
};
use std::{any::TypeId, ffi::CString};

const EXTERNAL_DOC: &str = include_str!("docs/external.md");
const CLASS_DOC: &str = concat!(
    "Inline comment.\nLiteral attribute.\n",
    include_str!("docs/external.md"),
    "\nConcatenated text.\nFinal comment."
);

#[gen_stub_pyclass]
#[pyclass(module = "doc_attributes")]
/// Inline comment.
#[doc = " Literal attribute."]
#[doc = include_str!("docs/external.md")]
#[doc = concat!("Concatenated ", "text.")]
/// Final comment.
struct Example {
    #[pyo3(get, set)]
    #[doc = include_str!("docs/external.md")]
    value: u32,
}

#[gen_stub_pymethods]
#[pymethods]
impl Example {
    #[doc = include_str!("docs/external.md")]
    fn answer(&self) -> u32 {
        42
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
#[doc = concat!("    Indented ", "macro output.")]
#[doc = concat!("Last ", "macro.")]
fn answer() -> u32 {
    42
}

#[gen_stub_pyclass_enum]
#[pyclass(module = "doc_attributes", skip_from_py_object)]
#[derive(Clone)]
#[doc = include_str!("docs/external.md")]
enum Simple {
    #[doc = concat!("Variant ", "docs.")]
    First,
}

#[gen_stub_pyclass_complex_enum]
#[pyclass(module = "doc_attributes", skip_from_py_object)]
#[derive(Clone)]
#[doc = include_str!("docs/external.md")]
enum Complex {
    #[doc = concat!("Structured variant ", "docs.")]
    Value {
        #[doc = include_str!("docs/external.md")]
        value: u32,
    },
}

#[test]
fn inventory_docs_match_runtime_docs() -> PyResult<()> {
    let class = inventory::iter::<PyClassInfo>
        .into_iter()
        .find(|info| (info.struct_id)() == TypeId::of::<Example>())
        .unwrap();
    let methods = inventory::iter::<PyMethodsInfo>
        .into_iter()
        .find(|info| (info.struct_id)() == TypeId::of::<Example>())
        .unwrap();
    let method = methods.methods.iter().find(|m| m.name == "answer").unwrap();
    let function = inventory::iter::<PyFunctionInfo>
        .into_iter()
        .find(|info| info.name == "answer")
        .unwrap();

    assert_eq!(class.doc, CLASS_DOC);
    assert_eq!(method.doc, EXTERNAL_DOC);
    assert_eq!(
        function.doc,
        format!("Free function docs.\n\n{EXTERNAL_DOC}\n    Indented macro output.\nLast macro.")
    );
    assert_eq!(class.getters[0].doc, EXTERNAL_DOC);
    assert_eq!(class.setters[0].doc, EXTERNAL_DOC);
    assert_eq!(methods.getters[0].doc, "Getter docs.");
    assert_eq!(methods.setters[0].doc, "Setter docs.");
    assert_eq!(
        methods
            .attrs
            .iter()
            .find(|m| m.name == "LIMIT")
            .unwrap()
            .doc,
        "Constant docs."
    );
    assert_eq!(
        methods
            .attrs
            .iter()
            .find(|m| m.name == "description")
            .unwrap()
            .doc,
        EXTERNAL_DOC
    );

    Python::initialize();
    Python::attach(|py| {
        let cls = py.get_type::<Example>();
        let runtime_class: String = cls.getattr("__doc__")?.extract()?;
        let runtime_method: String = cls.getattr("answer")?.getattr("__doc__")?.extract()?;
        let runtime_function: String = wrap_pyfunction!(answer, py)?
            .getattr("__doc__")?
            .extract()?;
        let runtime_field: String = cls.getattr("value")?.getattr("__doc__")?.extract()?;
        assert_eq!(runtime_class, class.doc);
        assert_eq!(runtime_method, method.doc);
        assert_eq!(runtime_function, function.doc);
        assert_eq!(runtime_field, class.getters[0].doc);

        // Exercise the doctest from the same external Markdown at runtime.
        let doctest = py.import("doctest")?;
        let test = doctest.getattr("DocTestParser")?.call0()?.call_method1(
            "get_doctest",
            (
                runtime_method,
                pyo3::types::PyDict::new(py),
                "answer",
                "external.md",
                0,
            ),
        )?;
        let result = doctest
            .getattr("DocTestRunner")?
            .call0()?
            .call_method1("run", (test,))?;
        assert_eq!(result.getattr("failed")?.extract::<usize>()?, 0);
        assert_eq!(result.getattr("attempted")?.extract::<usize>()?, 1);
        Ok(())
    })
}

#[test]
fn enum_and_variant_docs_preserve_expressions() {
    let simple = inventory::iter::<PyEnumInfo>
        .into_iter()
        .find(|info| (info.enum_id)() == TypeId::of::<Simple>())
        .unwrap();
    assert_eq!(simple.doc, EXTERNAL_DOC);
    assert_eq!(simple.variants, &[("First", "Variant docs.")]);

    let complex = inventory::iter::<PyComplexEnumInfo>
        .into_iter()
        .find(|info| (info.enum_id)() == TypeId::of::<Complex>())
        .unwrap();
    assert_eq!(complex.doc, EXTERNAL_DOC);
    assert_eq!(complex.variants[0].doc, "Structured variant docs.");
    assert_eq!(complex.variants[0].fields[0].doc, EXTERNAL_DOC);
}

#[test]
fn docs_reach_stubs_and_api_metadata() -> anyhow::Result<()> {
    let info = StubInfo::from_project_root(
        "doc_attributes".into(),
        env!("CARGO_MANIFEST_DIR").into(),
        false,
        Default::default(),
    )?;
    let stub = info.modules["doc_attributes"].to_string();
    let package = DocPackageBuilder::new(&info).build()?;
    let items = &package.modules["doc_attributes"].items;
    let class = items
        .iter()
        .find_map(|item| match item {
            DocItem::Class(class) if class.name == "Example" => Some(class),
            _ => None,
        })
        .unwrap();
    let function = items
        .iter()
        .find_map(|item| match item {
            DocItem::Function(function) if function.name == "answer" => Some(function),
            _ => None,
        })
        .unwrap();
    let method = class.methods.iter().find(|m| m.name == "answer").unwrap();
    assert_eq!(class.doc, CLASS_DOC);
    assert_eq!(method.doc, EXTERNAL_DOC.trim_end());
    assert!(function.doc.contains("    Indented macro output."));

    Python::initialize();
    Python::attach(|py| -> PyResult<()> {
        // Parse the actual rendered stub and compare its effective docstrings to
        // the API metadata, including Markdown table and doctest indentation.
        let locals = pyo3::types::PyDict::new(py);
        locals.set_item("stub", stub)?;
        locals.set_item("class_doc", &class.doc)?;
        locals.set_item("method_doc", &method.doc)?;
        locals.set_item("function_doc", &function.doc)?;
        py.run(
            &CString::new(
                r#"
import ast
tree = ast.parse(stub)
cls = next(n for n in tree.body if isinstance(n, ast.ClassDef) and n.name == 'Example')
method = next(n for n in cls.body if isinstance(n, ast.FunctionDef) and n.name == 'answer')
function = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == 'answer')
assert ast.get_docstring(cls) == class_doc
assert ast.get_docstring(method) == method_doc
assert ast.get_docstring(function) == function_doc
"#,
            )
            .unwrap(),
            Some(&locals),
            None,
        )
    })?;
    Ok(())
}
