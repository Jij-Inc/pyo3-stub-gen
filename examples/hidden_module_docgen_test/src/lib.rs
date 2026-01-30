use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::*};

// Classes that can be cross-referenced between modules (from hidden_module_docgen_test_import_type)
#[gen_stub_pyclass]
#[pyclass(module = "hidden_module_docgen_test._core")]
#[derive(Debug, Clone)]
struct A {
    x: usize,
}

#[gen_stub_pymethods]
#[pymethods]
impl A {
    fn show_x(&self) {
        println!("x = {}", self.x);
    }
}

#[gen_stub_pyfunction(module = "hidden_module_docgen_test._core")]
#[pyfunction]
fn create_a(x: usize) -> A {
    A { x }
}

#[gen_stub_pyfunction(module = "hidden_module_docgen_test._core")]
#[pyfunction]
pub fn wrap_opt_a(x: Option<A>) -> Option<A> {
    x
}

// Class without explicit module specification
#[gen_stub_pyclass]
#[pyclass(module = "hidden_module_docgen_test._core")]
#[derive(Debug, Clone)]
struct B {
    x: usize,
}

#[gen_stub_pymethods]
#[pymethods]
impl B {
    fn show_x(&self) {
        println!("x = {}", self.x);
    }
}

#[gen_stub_pyfunction]
#[pyfunction]
/// This is the docstring fo {py:func}`great_main` function.
///
/// These lines must be rendered as separate paragraphs.
///
/// ```python
/// >>> 42
/// 42
/// ```
///
/// This must be rendered: $x + x$
fn create_b(x: usize) -> B {
    B { x }
}

// Type alias in mod_a to test wildcard re-export
pyo3_stub_gen::type_alias!("hidden_module_docgen_test._core", ModAAlias = A);

// Type alias in mod_a to test wildcard re-export
pyo3_stub_gen::type_alias!("hidden_module_docgen_test._core", AorB = A | B);

#[gen_stub_pyfunction(module = "hidden_module_docgen_test._core")]
#[pyfunction(name = "greet_main")]
#[doc = r#"
    This is the docstring fo {py:func}`great_main` function.

    These lines must be rendered as separate paragraphs.

    ```python
    >>> 42
    42
    ```

    Another math test $\int_{-\infty}^\infty e^{-x^2} \mathrm{d}x$.
"#]
pub fn greet_main() {
    println!("Hello from main_mod!")
}

#[pymodule]
fn hidden_module_docgen_test(m: &Bound<PyModule>) -> PyResult<()> {
    // Add classes and functions to main module
    m.add_class::<A>()?;
    m.add_class::<B>()?;
    m.add_function(wrap_pyfunction!(create_a, m)?)?;
    m.add_function(wrap_pyfunction!(create_b, m)?)?;
    m.add_function(wrap_pyfunction!(greet_main, m)?)?;

    // Add submodules
    core(m)?;
    Ok(())
}

fn core(parent: &Bound<PyModule>) -> PyResult<()> {
    let py = parent.py();
    let sub = PyModule::new(py, "_core")?;
    sub.add_class::<A>()?;
    sub.add_class::<B>()?;
    sub.add_function(wrap_pyfunction!(create_a, &sub)?)?;
    sub.add_function(wrap_pyfunction!(create_b, &sub)?)?;
    sub.add_function(wrap_pyfunction!(greet_main, &sub)?)?;
    parent.add_submodule(&sub)?;
    Ok(())
}

// Test cases for __all__ generation escape hatches

// Test 1: Wildcard re-export fromthe submodule
pyo3_stub_gen::reexport_module_members!(
    "hidden_module_docgen_test",
    "hidden_module_docgen_test._core"
);

pyo3_stub_gen::module_doc!(
    "hidden_module_docgen_test",
    r#"
    This is the main module docstring for hidden_module_docgen_test.
    These lines are trimmed appropriately.

        This must be indented by four spaces.
    "#
);

define_stub_info_gatherer!(stub_info);
