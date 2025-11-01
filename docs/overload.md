# Function Overloading Support

This document describes the function overloading feature for Python stub generation in pyo3-stub-gen.

## Table of Contents

- [Overview](#overview)
- [Background](#background)
- [Usage](#usage)
  - [Basic Syntax](#basic-syntax)
  - [Use Case 1: Adding Overloads](#use-case-1-adding-overloads)
  - [Use Case 2: Overloads Only](#use-case-2-overloads-only)
  - [Advanced Examples](#advanced-examples)
- [Implementation](#implementation)
  - [Index-Based Ordering](#index-based-ordering)
  - [Validation Rules](#validation-rules)
  - [Stub Generation](#stub-generation)
- [Testing](#testing)
- [Backward Compatibility](#backward-compatibility)

## Overview

pyo3-stub-gen supports Python's `@typing.overload` decorator for defining multiple type signatures for a single function. This is useful when:

1. A function accepts different types and returns different types based on the input
2. A function has optional parameters that affect the return type
3. A function uses `Literal` types to determine the return type

**Key Features:**

- **`python_overload` parameter**: Define multiple overload signatures inline with `#[gen_stub_pyfunction]`
- **`no_default_overload` flag**: Suppress auto-generation from Rust types
- **Automatic ordering**: Deterministic overload ordering using index-based sorting
- **Runtime validation**: Prevents inconsistent overload definitions
- **Backward compatible**: Existing `submit!` syntax continues to work

## Background

Python's type system allows multiple type signatures for a single function using the `@typing.overload` decorator. In `.pyi` stub files, all overload variants should have the `@overload` decorator:

```python
@overload
def func(x: int) -> int: ...

@overload
def func(x: float) -> float: ...
```

This provides precise type information to type checkers like pyright and mypy.

## Usage

### Basic Syntax (Functions)

**For module-level functions:**

```rust
#[gen_stub_pyfunction(
    python_overload = r#"
    @overload
    def function_name(param: Type1) -> ReturnType1: ...

    @overload
    def function_name(param: Type2) -> ReturnType2: ...
    "#
)]
#[pyfunction]
pub fn function_name(param: RustType) -> RustReturnType {
    // Implementation
}
```

### Use Case 1: Adding Overloads

Add overload signatures while **also generating** a type hint from the Rust function signature.

**Example**: `overload_example_1` from `examples/pure/src/overloading.rs`

```rust
#[gen_stub_pyfunction(
    python_overload = r#"
    @overload
    def overload_example_1(x: int) -> int: ...
    "#
)]
#[pyfunction]
pub fn overload_example_1(x: f64) -> f64 {
    x + 1.0
}
```

**Generated stub** (`pure.pyi`):

```python
@overload
def overload_example_1(x: int) -> int: ...

@overload
def overload_example_1(x: float) -> float: ...  # Auto-generated from Rust
```

**Note**: Both signatures get the `@overload` decorator. The Rust signature (`f64 -> f64`) is automatically added as an additional overload variant.

### Use Case 2: Overloads Only

Define **only** overload signatures without generating from the Rust signature.

**Example**: `overload_example_2` from `examples/pure/src/overloading.rs`

```rust
#[gen_stub_pyfunction(
    python_overload = r#"
    @overload
    def overload_example_2(ob: int) -> int:
        """Increments integer by 1"""

    @overload
    def overload_example_2(ob: float) -> float:
        """Increments float by 1"""
    "#,
    no_default_overload = true  // Don't generate from Rust signature
)]
#[pyfunction]
pub fn overload_example_2(ob: Bound<PyAny>) -> PyResult<PyObject> {
    // Runtime type checking
    let py = ob.py();
    if let Ok(f) = ob.extract::<f64>() {
        (f + 1.0).into_py_any(py)
    } else if let Ok(i) = ob.extract::<i64>() {
        (i + 1).into_py_any(py)
    } else {
        Err(PyTypeError::new_err("Invalid type"))
    }
}
```

**Generated stub** (`pure.pyi`):

```python
@overload
def overload_example_2(ob: int) -> int:
    """Increments integer by 1"""

@overload
def overload_example_2(ob: float) -> float:
    """Increments float by 1"""
```

**Note**: The `no_default_overload = true` flag suppresses automatic generation from the Rust signature. This is useful when the Rust types (like `Bound<PyAny>`) don't provide useful type information for Python users.

### Advanced Examples

#### Literal-based Overloading

Use `typing.Literal` to define return types based on literal parameter values:

```rust
#[gen_stub_pyfunction(
    python_overload = r#"
    import typing
    import collections.abc

    @overload
    def as_tuple(xs: collections.abc.Sequence[int], /, *, tuple_out: typing.Literal[True]) -> tuple[int, ...]:
        """Convert sequence to tuple when tuple_out is True"""

    @overload
    def as_tuple(xs: collections.abc.Sequence[int], /, *, tuple_out: typing.Literal[False]) -> list[int]:
        """Convert sequence to list when tuple_out is False"""
    "#,
    no_default_overload = true
)]
#[pyfunction]
#[pyo3(signature = (xs, /, *, tuple_out))]
pub fn as_tuple(xs: Vec<i32>, tuple_out: bool) -> PyResult<PyObject> {
    Python::with_gil(|py| {
        if tuple_out {
            Ok(PyTuple::new(py, xs.iter())?.into_py_any(py)?)
        } else {
            Ok(xs.into_py_any(py)?)
        }
    })
}
```

**Generated stub**:

```python
@overload
def as_tuple(xs: collections.abc.Sequence[int], /, *, tuple_out: typing.Literal[True]) -> tuple[int, ...]:
    """Convert sequence to tuple when tuple_out is True"""

@overload
def as_tuple(xs: collections.abc.Sequence[int], /, *, tuple_out: typing.Literal[False]) -> list[int]:
    """Convert sequence to list when tuple_out is False"""
```

### Class Method Overloads

Class methods support overloading using `gen_methods_from_python!`. The basic pattern manually submits method signatures:

**Example**: `ManualSubmit` from `examples/pure/src/manual_submit.rs`

```rust
use pyo3::prelude::*;
use pyo3_stub_gen::{derive::*, inventory::submit};

#[gen_stub_pyclass]
#[pyclass]
pub struct ManualSubmit {}

// Manually submit method info for the class
submit! {
    gen_methods_from_python! {
        r#"
        class ManualSubmit:
            def __new__(cls) -> ManualSubmit:
                """Constructor for ManualSubmit class"""
                ...

            def increment(self, x: float) -> float:
                """Add 1.0 to the input float"""
                ...

            @overload
            def echo(self, obj: int) -> int:
                """If the input is an int, returns int"""

            @overload
            def echo(self, obj: float) -> float:
                """If the input is a float, returns float"""
        "#
    }
}

#[pymethods]
impl ManualSubmit {
    #[new]
    fn new() -> Self {
        ManualSubmit {}
    }

    fn increment(&self, x: f64) -> f64 {
        x + 1.0
    }

    fn echo<'arg>(&self, obj: Bound<'arg, PyAny>) -> Bound<'arg, PyAny> {
        obj
    }
}
```

**Generated stub** (`pure.pyi`):

```python
class ManualSubmit:
    def __new__(cls) -> ManualSubmit:
        """Constructor for ManualSubmit class"""

    def increment(self, x: float) -> float:
        """Add 1.0 to the input float"""

    @typing.overload
    def echo(self, obj: int) -> int:
        """If the input is an int, returns int"""

    @typing.overload
    def echo(self, obj: float) -> float:
        """If the input is a float, returns float"""
```

**Note**: The same validation rules apply to class methods:
- Multiple methods with the same name must have `@overload` decorator
- At most one method can have `is_overload = false` (the implementation)
- The `@overload` decorator is automatically detected from Python stub syntax

### Advanced Class Method Patterns

For more complex scenarios, you can use manual submission patterns. See `examples/pure/src/manual_submit.rs` for complete examples.

#### Pattern 1: Fully Manual Method Submission

When you need complete control over all method signatures, submit them manually without using `#[gen_stub_pymethods]`:

```rust
use pyo3::prelude::*;
use pyo3_stub_gen::{derive::*, inventory::submit};

/// Demonstrates manual submission of class methods using the `submit!` macro
#[gen_stub_pyclass] // Use proc-macro for submitting class info
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
    gen_methods_from_python! {
        r#"
        class ManualSubmit:
            def __new__(cls) -> ManualSubmit:
                """Constructor for ManualSubmit class"""
                ...

            def increment(self, x: float) -> float:
                """Add 1.0 to the input float"""
                ...

            @overload
            def echo(self, obj: int) -> int:
                """If the input is an int, returns int"""

            @overload
            def echo(self, obj: float) -> float:
                """If the input is a float, returns float"""
        "#
    }
}
```

**Generated stub** (`pure.pyi`):

```python
class ManualSubmit:
    def __new__(cls) -> ManualSubmit:
        """Constructor for ManualSubmit class"""

    def increment(self, x: float) -> float:
        """Add 1.0 to the input float"""

    @typing.overload
    def echo(self, obj: int) -> int:
        """If the input is an int, returns int"""

    @typing.overload
    def echo(self, obj: float) -> float:
        """If the input is a float, returns float"""
```

**When to use:**
- Need complete control over all method signatures
- Want to define overloads that don't match Rust implementation exactly
- Building code generation tools or macros

#### Pattern 2: Mixing Proc-Macro and Manual Submission

For classes where most methods can use automatic generation but some need manual signatures:

```rust
use pyo3::prelude::*;
use pyo3_stub_gen::{derive::*, inventory::submit};

#[gen_stub_pyclass]
#[pyclass]
pub struct PartialManualSubmit {}

// Manually submit method info for the `PartialManualSubmit` class.
//
// IMPORTANT: The `submit!` invocation must appear BEFORE the `#[gen_stub_pymethods]` impl block
// when including `@overload` entries, because Python overload resolution depends on definition
// order and pyo3-stub-gen orders them by source position.
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

#[gen_stub_pymethods]
#[pymethods]
impl PartialManualSubmit {
    #[new]
    fn new() -> Self {
        PartialManualSubmit {}
    }

    fn echo<'arg>(&self, obj: Bound<'arg, PyAny>) -> Bound<'arg, PyAny> {
        obj
    }

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
```

**Generated stub** (`pure.pyi`):

```python
class PartialManualSubmit:
    @typing.overload
    def echo_overloaded(self, obj: int) -> int:
        """Overloaded version for int input"""

    @typing.overload
    def echo_overloaded(self, obj: typing.Any) -> typing.Any: ...

    def fn_override_type(self, cb: collections.abc.Callable[[str], typing.Any]) -> collections.abc.Callable[[str], typing.Any]:
        """Example method with complex type annotation, skipped from #[gen_stub_pymethods]"""

    def __new__(cls) -> PartialManualSubmit: ...

    def echo(self, obj: typing.Any) -> typing.Any: ...
```

**Key points:**
- ✅ Use `#[gen_stub(skip)]` to skip methods that need manual type annotations
- ✅ Place `submit!` blocks **BEFORE** the `#[gen_stub_pymethods]` impl block for proper overload ordering
- ✅ Combine automatic generation for simple methods with manual submission for complex cases

**When to use:**
- Most methods work with automatic generation
- A few methods need complex type annotations (e.g., `Callable`)
- Need to add overloads to auto-generated methods

## Implementation

### Index-Based Ordering

When multiple overload variants are generated from a single macro invocation, they share the same source location (`file`, `line`, `column`). To ensure deterministic ordering, each overload is assigned a sequential index (0, 1, 2, ...).

**Ordering Key**: `(file, line, column, index)`

This guarantees that overloads appear in the `.pyi` file in the same order they were defined:

```rust
#[gen_stub_pyfunction(
    python_overload = r#"
    @overload
    def func(x: int) -> int: ...      // index = 0

    @overload
    def func(x: float) -> float: ...  // index = 1
    "#
)]
#[pyfunction]
pub fn func(x: f64) -> f64 { x }      // index = 2 (if auto-generated)
```

The generated `PyFunctionInfo` structures include:

```rust
PyFunctionInfo {
    // ... other fields ...
    is_overload: true,
    file: "src/example.rs",
    line: 10,
    column: 1,
    index: 0,  // Sequential index for deterministic ordering
}
```

At stub generation time, functions are sorted by `(file, line, column, index)`, ensuring:
- Functions from different source locations are ordered by file location
- Functions from the same source location are ordered by their index

### Validation Rules

**Compile-time (Proc Macro):**

1. **Function names must match**: All functions in `python_overload` must have the same name as the Rust function
2. **All must have `@overload`**: In `python_overload`, all functions must have the `@overload` decorator
3. **Cannot mix parameters**: Error if both `python` and `python_overload` are specified

**Runtime (Stub Generation):**

These rules apply to **both module-level functions and class methods**:

1. **Multiple non-overload functions/methods**: If 2+ functions/methods with the same name have `is_overload = false`, this is an error:
   ```
   Error: Multiple functions with name 'func' found without @overload decorator.
   Please add @overload decorator to all variants.
   ```

   For class methods:
   ```
   Error: Multiple methods with name 'method' in class 'ClassName' found without @overload decorator.
   Please add @overload decorator to all variants.
   ```

2. **Overload propagation**: If any function/method in a same-name group has `is_overload = true`, all functions/methods get `@overload` in the generated stub

3. **Allowed patterns**:
   - Single function/method (no overload): `is_overload = false`
   - Multiple overload-only: All have `is_overload = true`
   - Mixed (one implementation + overloads): At most one `is_overload = false` + any number of `is_overload = true`

### Stub Generation

The stub generation logic applies to both functions (`pyo3-stub-gen/src/generate/module.rs`) and methods (`pyo3-stub-gen/src/generate/class.rs`).

**For module-level functions** (`module.rs`):

```rust
// Validation: Check for multiple non-overload functions (error case)
let non_overload_count = functions.iter().filter(|f| !f.is_overload).count();
if non_overload_count > 1 {
    panic!(
        "Multiple functions with name '{}' found without @overload decorator. \
         Please add @overload decorator to all variants.",
        function_name
    );
}

// Check if we should add @overload to all functions
let has_overload = functions.iter().any(|f| f.is_overload);
let should_add_overload = functions.len() > 1 && has_overload;

// Sort by source location and index for deterministic ordering
let mut sorted_functions = functions.clone();
sorted_functions.sort_by_key(|func| (func.file, func.line, func.column, func.index));

for function in sorted_functions {
    if should_add_overload {
        writeln!(f, "@typing.overload")?;
    }
    write!(f, "{function}")?;
}
```

**Key behaviors**:
- Functions are sorted by `(file, line, column, index)` before generation
- If any function has `is_overload = true` and there are multiple functions, all get `@overload`
- Validates that at most one function has `is_overload = false`

**For class methods** (`class.rs`):

```rust
for (method_name, methods) in &self.methods {
    // Validation: Check for multiple non-overload methods (error case)
    let non_overload_count = methods.iter().filter(|m| !m.is_overload).count();
    if non_overload_count > 1 {
        panic!(
            "Multiple methods with name '{}' in class '{}' found without @overload decorator. \
             Please add @overload decorator to all variants.",
            method_name, self.name
        );
    }

    // Check if we should add @overload to all methods
    let has_overload = methods.iter().any(|m| m.is_overload);
    let should_add_overload = methods.len() > 1 && has_overload;

    for method in methods {
        if should_add_overload {
            writeln!(f, "{indent}@typing.overload")?;
        }
        method.fmt(f)?;
    }
}
```

**Key behaviors** (same as functions):
- If any method has `is_overload = true` and there are multiple methods, all get `@overload`
- Validates that at most one method has `is_overload = false`
- Methods within a class are not sorted (they maintain the order from `IndexMap`)

## Testing

### End-to-End Tests (examples/pure)

**pytest**: ✅ All 21 tests passed (2025-11-01)

```bash
$ cd examples/pure && uv run pytest
============================== 21 passed in 0.75s ==============================
```

Key tests:
- `test_overload_example_1`: Tests `python_overload` with auto-generated Rust signature
- `test_overload_example_2`: Tests `python_overload` with `no_default_overload`
- `test_as_tuple`: Tests Literal-based overloading

**pyright**: ✅ 0 errors (2025-11-01)

```bash
$ cd examples/pure && uv run pyright
0 errors, 0 warnings, 0 informations
```

The overload ordering issue has been fixed, and pyright no longer reports overlapping overload errors.

### Proc-Macro Level Tests (pyo3-stub-gen-derive)

**cargo test**: ✅ All 53 tests passed (2025-11-01)

Includes 3 macro expansion snapshot tests:

1. **`test_overload_example_1_expansion`**: Verifies that `python_overload` + auto-generated produces two `inventory::submit!` blocks with correct `index` values (0, 1)

2. **`test_overload_example_2_expansion`**: Verifies that `python_overload` + `no_default_overload` produces only the manually-defined overloads

3. **`test_regular_function_no_overload`**: Verifies that regular functions have `is_overload: false` and `index: 0`

Plus 50 parsing-level tests for Python stub syntax parsing.

### Rust Tests

**cargo test**: ✅ All workspace tests pass (2025-11-01)

```bash
$ cargo test
   Doc-tests pyo3_stub_gen_derive: 8 passed
   Doc-tests pyo3_stub_gen: 20 passed
```

## Backward Compatibility

### Existing `submit!` Syntax

The existing `submit! { gen_function_from_python! { ... } }` syntax continues to work:

```rust
submit! {
    gen_function_from_python! {
        r#"
        @overload
        def func(x: int) -> int: ...
        "#
    }
}

#[gen_stub_pyfunction]
#[pyfunction]
pub fn func(x: f64) -> f64 { x }
```

**Behavior**: The `@overload` decorator in the Python code sets `is_overload = true`, and the auto-generated signature from Rust also gets `@overload` in the final stub.

### Migration from Old Code

**Old pattern** (still works):
```rust
submit! {
    gen_function_from_python! { r#"
        @overload
        def func(x: int) -> int: ...
    "# }
}

#[gen_stub_pyfunction]
#[pyfunction]
pub fn func(x: f64) -> f64 { x }
```

**New pattern** (recommended):
```rust
#[gen_stub_pyfunction(
    python_overload = r#"
    @overload
    def func(x: int) -> int: ...
    "#
)]
#[pyfunction]
pub fn func(x: f64) -> f64 { x }
```

Both generate the same output:
```python
@overload
def func(x: int) -> int: ...

@overload
def func(x: float) -> float: ...
```

### Breaking Change Warning

**Old behavior** (before this implementation):
- Multiple functions/methods with the same name automatically get `@overload` decorator
- No error even if `@overload` is missing

**New behavior** (after this implementation):
- Multiple functions/methods with the same name AND all without `@overload` → **Error**
- Must explicitly add `@overload` decorator

**Migration**:
- For functions: Add `@overload` decorator to all manual overload definitions in `gen_function_from_python!` calls
- For methods: Add `@overload` decorator to all manual overload definitions in `gen_methods_from_python!` calls
