# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Python stub file (*.pyi) generator for PyO3-based Rust projects. It automatically generates Python type hints for Rust code exposed to Python through PyO3, enabling better IDE support and type checking for mixed Rust/Python projects.

## Architecture

### Workspace Structure
- `pyo3-stub-gen/` - Core library for stub generation
- `pyo3-stub-gen-derive/` - Procedural macros for automatic metadata collection  
- `examples/` - Example projects showing different maturin layouts (pure, mixed, mixed_sub, etc.)

### Three-Phase Generation Process
1. **Compile-time**: Procedural macros (`#[gen_stub_pyclass]`, `#[gen_stub_pyfunction]`) extract type information using the `inventory` crate
2. **Runtime**: `define_stub_info_gatherer!` macro collects metadata and reads `pyproject.toml` configuration
3. **Generation**: Transforms metadata into Python stub syntax and generates `.pyi` files

### Procedural Macro Design Pattern (`pyo3-stub-gen-derive`)

The derive crate follows a consistent three-layer architecture:

1. **Entry Point (`src/gen_stub.rs`)**:
   - Public functions handle `TokenStream` parsing and generation
   - Examples: `pyclass()`, `pyfunction()`, `gen_function_from_python_impl()`
   - This is the ONLY module that directly manipulates `TokenStream`

2. **Intermediate Representation (`src/gen_stub/*.rs`)**:
   - Each module provides `*Info` structs (e.g., `PyFunctionInfo`, `PyClassInfo`)
   - Conversion from `syn` types: `TryFrom<ItemFn>`, `TryFrom<ItemStruct>`, etc.
   - Implementation of `ToTokens` trait for code generation

3. **Flow Pattern**:
   ```rust
   // In gen_stub.rs
   pub fn pyfunction(attr: TokenStream2, item: TokenStream2) -> Result<TokenStream2> {
       let item_fn = parse2::<ItemFn>(item)?;           // Parse TokenStream
       let inner = PyFunctionInfo::try_from(item_fn)?;  // Convert to Info struct
       Ok(quote! { #inner })                             // Generate via ToTokens
   }
   ```

**Important**: When adding new functionality, follow this pattern strictly:
- TokenStream manipulation stays in `gen_stub.rs`
- Business logic and intermediate representations go in `gen_stub/*.rs`
- Use `ToTokens` trait for code generation, not direct `quote!` in submodules

## Development Commands

### Build and Testing
```bash
# Generate stub files for all examples
task stub-gen

# Run comprehensive tests (pytest + pyright + ruff) for all examples  
task test

# Work with specific example (replace 'pure' with mixed, mixed_sub, etc.)
task pure:stub-gen
task pure:test
```

### Individual Example Commands
```bash
cd examples/pure  # or mixed, mixed_sub, etc.

# Generate stub files
cargo run --bin stub_gen

# Run Python tests and type checking
uv run pytest
uv run pyright
uvx ruff check
uv run mypy --show-error-codes -p <module_name>
uv run stubtest <module_name> --ignore-missing-stub --ignore-disjoint-bases
```

### Core Development
```bash
# Build the workspace
cargo build

# Run Rust tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy
```

## Key Requirements

### Cargo.toml Configuration
All PyO3 projects must include both `cdylib` and `rlib` crate types:
```toml
[lib]
crate-type = ["cdylib", "rlib"]
```

### Stub Generation Binary
Each project needs a `src/bin/stub_gen.rs` executable:
```rust
use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    let stub = your_crate::stub_info()?;
    stub.generate()?;
    Ok(())
}
```

## Procedural Macro Usage

### Core Macros
- `#[gen_stub_pyclass]` - For PyO3 classes
- `#[gen_stub_pyfunction]` - For PyO3 functions  
- `#[gen_stub_pymethods]` - For PyO3 method implementations
- `#[gen_stub_pyclass_enum]` - For PyO3 enums

### Required Placement
Proc-macros MUST be placed before corresponding PyO3 macros:
```rust
#[gen_stub_pyfunction]  // Must come first
#[pyfunction]
fn my_function() { ... }
```

### Information Gathering
Each crate must define:
```rust
use pyo3_stub_gen::define_stub_info_gatherer;
define_stub_info_gatherer!(stub_info);
```

### Python Stub Syntax Support

For complex type definitions (e.g., `collections.abc.Callable`, overloads, generics), you can write type information directly in Python stub syntax instead of using Rust attributes.

#### Three Approaches

**1. Inline Python Parameter (Recommended for single functions)**

Use the `python` parameter in `#[gen_stub_pyfunction]` to specify types directly:

```rust
#[gen_stub_pyfunction(python = r#"
    import collections.abc
    import typing

    def fn_with_python_param(callback: collections.abc.Callable[[str], typing.Any]) -> collections.abc.Callable[[str], typing.Any]:
        """Example using python parameter."""
"#)]
#[pyfunction]
pub fn fn_with_python_param<'a>(callback: Bound<'a, PyAny>) -> PyResult<Bound<'a, PyAny>> {
    callback.call1(("Hello!",))?;
    Ok(callback)
}
```

**2. Function Stub Generation Macro (For overloads or separate definitions)**

Use `gen_function_from_python!` inside `submit!` block:

```rust
// Rust implementation
#[pyfunction]
pub fn overload_example(x: f64) -> f64 {
    x + 1.0
}

// Additional overload definition
use pyo3_stub_gen::inventory::submit;

submit! {
    gen_function_from_python! {
        r#"
        def overload_example(x: int) -> int: ...
        "#
    }
}
```

**3. Methods Stub Generation Macro (For class methods)**

Use `gen_methods_from_python!` to define multiple method signatures at once:

```rust
#[gen_stub_pyclass]
#[pyclass]
pub struct Calculator {}

#[gen_stub_pymethods]
#[pymethods]
impl Calculator {
    fn add(&self, x: f64) -> f64 {
        x + 1.0
    }
}

// Additional overload for integer type
submit! {
    gen_methods_from_python! {
        r#"
        class Calculator:
            def add(self, x: int) -> int:
                """Add operation for integers"""
        "#
    }
}
```

#### When to Use

- **Complex types**: `collections.abc.Callable`, `typing.Protocol`, nested generics
- **Overloads**: Multiple type signatures for the same function (`@overload` in `.pyi`)
- **Type override**: When automatic Rust → Python type mapping is insufficient
- **Readability**: Python developers find stub syntax more familiar

#### Notes

- Python stub syntax is parsed at compile time using `rustpython-parser`
- Type information is stored as strings (no Rust type validation)
- Import statements are automatically extracted and included in generated `.pyi` files
- This approach complements automatic type generation, not replaces it

## Testing Strategy

Each example includes comprehensive testing:
- `pytest` - Python functionality tests
- `pyright` - Static type checking validation
- `ruff` - Python code linting
- `mypy` - Type checking with mypy
- `stubtest` - Validates stubs match runtime behavior (mypy.stubtest)
- Tests verify that generated stubs provide correct type information

### Stubtest Notes
- Always use `--ignore-missing-stub` flag (maturin creates internal native modules without stubs)
- Always use `--ignore-disjoint-bases` flag (PyO3 classes are disjoint bases at runtime)
- Stubtest does not work with nested submodules due to PyO3's runtime attribute approach

## Type System

The project provides mappings between Rust and Python types:
- `/stub_type/builtins.rs` - Basic Python types
- `/stub_type/collections.rs` - Python collections
- `/stub_type/numpy.rs` - NumPy array types
- `/stub_type/pyo3.rs` - PyO3-specific types

Manual type specification is supported for edge cases where automatic translation fails.

## Important Notes

- Python 3.10+ is the minimum supported version (do not enable 3.9 or older in PyO3 settings)
- When you think you are done with your change, use `task stub-gen` to update stub files and inspect the content
- The project uses the `inventory` crate for compile-time metadata collection across proc-macros
- Stub files are automatically included in wheel packages by maturin