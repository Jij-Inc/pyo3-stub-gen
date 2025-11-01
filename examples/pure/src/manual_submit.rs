use pyo3::prelude::*;
use pyo3_stub_gen::{derive::*, inventory::submit};

/// Demonstrates manual submission of class methods using the `submit!` macro
#[gen_stub_pyclass] // Use proc-macro for submitting class info
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[pyclass]
pub struct ManualSubmit {}

// No #[gen_stub_pymethods]
// i.e., the following methods will not appear in the stub unless we manually submit them
#[pymethods]
impl ManualSubmit {
    #[new]
    fn new() -> Self {
        ManualSubmit {}
    }

    fn increment(&self, x: f64) -> f64 {
        x + 1.0
    }

    // Returns the input object as is
    fn echo<'arg>(&self, obj: Bound<'arg, PyAny>) -> Bound<'arg, PyAny> {
        obj
    }
}

// Manually submit method info for the `ManualSubmit` class.
submit! {
    // Generator macro to create `pyo3_stub_gen::PyMethodsInfo` from a Python code snippet
    gen_methods_from_python! {
        r#"
        #
        # This is Python code. We can write Python comments here.
        #

        # The class name must match the Rust struct name.
        class ManualSubmit:
            def __new__(cls) -> ManualSubmit:
                """Constructor for ManualSubmit class"""
                ...

            def increment(self, x: float) -> float:
                """Add 1.0 to the input float"""
                ...

            #
            # Using manual submission, we can write @overload decorators for the `echo` method.
            #
            # Since Python's overload resolution depends on the order of definitions,
            # we write the more specific type (int) first.
            #

            @overload
            def echo(self, obj: int) -> int:
                """If the input is an int, returns int"""

            @overload
            def echo(self, obj: float) -> float:
                """If the input is a float, returns float"""
        "#
    }
}

/// Example demonstrating manual submission mixed with proc-macro generated method info
#[gen_stub_pyclass] // Use proc-macro for submitting class info
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[pyclass]
pub struct PartialManualSubmit {}

// Manually submit method info for the `PartialManualSubmit` class.
//
// Since we also use `#[gen_stub_pymethods]` for this class, what we should submit here are only:
// - `@overload` entries
// - Complex type annotations that cannot be expressed in the Rust type system for `#[gen_stub(skip)]`-ed methods
//
// Note
// ----
// The `submit!` invocation must appear before the `#[gen_stub_pymethods]` impl block when including `@overload` entries,
// because Python overload resolution depends on definition order and pyo3-stub-gen orders them by source position.
submit! {
    gen_methods_from_python! {
        r#"
        import typing
        import collections.abc

        class PartialManualSubmit:
            @overload
            def echo_overloaded(self, obj: int) -> int:
                """Overloaded version for int input"""

            def fn_override_type(self, cb: collections.abc.Callable[[str], typing.Any]) -> collections.abc.Callable[[str], typing.Any]:
                """Example method with complex type annotation, skipped from #[gen_stub_pymethods]"""
        "#
    }
}

/// Generates method info for the `PartialManualSubmit` class using proc-macro
#[gen_stub_pymethods]
#[pymethods]
impl PartialManualSubmit {
    /// The constructor for PartialManualSubmit
    #[new]
    fn new() -> Self {
        PartialManualSubmit {}
    }

    // Returns the input object as is
    fn echo<'arg>(&self, obj: Bound<'arg, PyAny>) -> Bound<'arg, PyAny> {
        obj
    }

    // Returns the input object as is (overloaded)
    fn echo_overloaded<'arg>(&self, obj: Bound<'arg, PyAny>) -> Bound<'arg, PyAny> {
        obj
    }

    /// Method with complex type annotation, skipped from #[gen_stub_pymethods]
    #[gen_stub(skip)]
    pub fn fn_override_type<'a>(&self, cb: Bound<'a, PyAny>) -> PyResult<Bound<'a, PyAny>> {
        cb.call1(("Hello!",))?;
        Ok(cb)
    }
}
