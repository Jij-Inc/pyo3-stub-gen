use pyo3::prelude::*;
use pyo3_stub_gen::{
    define_stub_info_gatherer, derive::*, runtime::PyModuleTypeAliasExt, type_alias,
};

// Example classes and functions for generate-init-py demonstration
#[gen_stub_pyclass]
#[pyclass(module = "generate_init_py._core")]
#[derive(Debug, Clone)]
pub struct A {
    x: usize,
}

#[gen_stub_pymethods]
#[pymethods]
impl A {
    fn show_x(&self) {
        println!("x = {}", self.x);
    }
}

#[gen_stub_pyfunction(module = "generate_init_py._core")]
#[pyfunction]
fn create_a(x: usize) -> A {
    A { x }
}

#[gen_stub_pyfunction(module = "generate_init_py._core")]
#[pyfunction]
pub fn wrap_opt_a(x: Option<A>) -> Option<A> {
    x
}

// Class without explicit module specification
#[gen_stub_pyclass]
#[pyclass(module = "generate_init_py._core")]
#[derive(Debug, Clone)]
struct B {
    x: usize,
}

// Class without explicit module specification
#[gen_stub_pyclass_enum]
#[pyclass(module = "generate_init_py._core", frozen, eq)]
#[derive(Debug, Clone, PartialEq, Eq)]
/// This is the docstring for enum C.
enum C {
    /// This is 0th variant
    C0,
    /// This is 1st variant
    C1,
    /// This is 2nd variant
    C2,
}

#[gen_stub_pymethods]
#[pymethods]
impl B {
    fn show_x(&self) {
        println!("x = {}", self.x);
    }
}

#[gen_stub_pyfunction(module = "generate_init_py._core")]
#[pyfunction]
/// This is the docstring for {py:func}`great_main` function.
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

/// A function with a default argument
#[gen_stub_pyfunction(module = "generate_init_py._core")]
#[pyfunction(signature = (c = C::C1))]
fn default_c(c: C) -> C {
    c
}

// Runtime type alias using type_alias! - available both in stubs AND at runtime
// This can be imported via the generated __init__.py
type_alias!(
    "generate_init_py._core",
    AorB = A | B,
    "A union type of A or B, available at runtime."
);

#[gen_stub_pyfunction(module = "generate_init_py._core")]
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

/// The main PyO3 module entry point (exposed as generate_init_py._core)
#[pymodule]
fn _core(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<A>()?;
    m.add_class::<B>()?;
    m.add_class::<C>()?;
    m.add_function(wrap_pyfunction!(create_a, m)?)?;
    m.add_function(wrap_pyfunction!(create_b, m)?)?;
    m.add_function(wrap_pyfunction!(greet_main, m)?)?;
    m.add_function(wrap_pyfunction!(wrap_opt_a, m)?)?;
    m.add_function(wrap_pyfunction!(default_c, m)?)?;
    // Register runtime type alias - now importable from Python
    m.add_type_alias::<AorB>()?;
    Ok(())
}

// Re-export all items from _core to the parent module (using new syntax)
pyo3_stub_gen::reexport_module_members!("generate_init_py" from "generate_init_py._core");

pyo3_stub_gen::module_doc!(
    "generate_init_py",
    r#"
    This is the main module docstring for generate_init_py.
    This example demonstrates the generate-init-py feature.

        This must be indented by four spaces.
    "#
);

define_stub_info_gatherer!(stub_info);
