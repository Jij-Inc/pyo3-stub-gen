//! Example demonstrating Python 3.12+ `type` statement syntax for type aliases.
//! This example tests the `use-type-statement = true` configuration option.

use pyo3::prelude::*;
use pyo3_stub_gen::define_stub_info_gatherer;
use std::collections::HashMap;

// Simple type aliases using the type_alias! macro
pyo3_stub_gen::type_alias!("type_statement_alias", SimpleAlias = Option<usize>);
pyo3_stub_gen::type_alias!("type_statement_alias", StrIntMap = HashMap<String, i32>);

// Union type aliases (direct union syntax)
pyo3_stub_gen::type_alias!("type_statement_alias", NumberOrString = i32 | String);
pyo3_stub_gen::type_alias!("type_statement_alias", TripleUnion = i32 | String | bool);
pyo3_stub_gen::type_alias!("type_statement_alias", GenericUnion = Option<i32> | Vec<String>);

// Complex nested type alias
pyo3_stub_gen::type_alias!(
    "type_statement_alias",
    ComplexNested = Option<HashMap<String, Vec<i32>>>
);

// Type alias using Python stub syntax with pre-3.12 syntax
// The parser accepts both syntaxes; output is controlled by use-type-statement config
pyo3_stub_gen::derive::gen_type_alias_from_python!(
    "type_statement_alias",
    r#"
    import collections.abc
    from typing import TypeAlias
    CallbackType: TypeAlias = collections.abc.Callable[[str], None]
    "#
);

// Type alias using Python stub syntax with 3.12+ type statement
// This tests that the parser accepts the new syntax
pyo3_stub_gen::derive::gen_type_alias_from_python!(
    "type_statement_alias",
    r#"
    import collections.abc
    type OptionalCallback = collections.abc.Callable[[str], None] | None
    "#
);

#[pymodule]
fn type_statement_alias(_m: &Bound<PyModule>) -> PyResult<()> {
    Ok(())
}

// Define stub info gatherer
define_stub_info_gatherer!(stub_info);
