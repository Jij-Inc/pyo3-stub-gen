//! Test for typing.Literal with boolean values

use pyo3::prelude::*;
use pyo3_stub_gen::{derive::*, inventory::submit};

/// Test function with Literal[True]
#[pyfunction]
pub fn returns_true() -> bool {
    true
}

submit! {
    gen_function_from_python! {
        r#"
        import typing
        def returns_true() -> typing.Literal[True]: ...
        "#
    }
}

/// Test function with Literal[False]
#[pyfunction]
pub fn returns_false() -> bool {
    false
}

submit! {
    gen_function_from_python! {
        r#"
        import typing
        def returns_false() -> typing.Literal[False]: ...
        "#
    }
}

/// Test function with Literal[True, False] (i.e., bool)
#[pyfunction]
pub fn returns_bool(value: bool) -> bool {
    value
}

submit! {
    gen_function_from_python! {
        r#"
        import typing
        def returns_bool(value: typing.Literal[True, False]) -> typing.Literal[True, False]: ...
        "#
    }
}

/// Test function with inline python parameter
#[gen_stub_pyfunction(python = r#"
    import typing
    def literal_true_inline() -> typing.Literal[True]: ...
"#)]
#[pyfunction]
pub fn literal_true_inline() -> bool {
    true
}

/// Test function with inline python parameter for False
#[gen_stub_pyfunction(python = r#"
    import typing
    def literal_false_inline() -> typing.Literal[False]: ...
"#)]
#[pyfunction]
pub fn literal_false_inline() -> bool {
    false
}
