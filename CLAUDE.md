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

> [!NOTE]
> For detailed architecture documentation, see [`docs/architecture.md`](./docs/architecture.md)

### Procedural Macro Design Pattern (`pyo3-stub-gen-derive`)

The derive crate follows a consistent three-layer architecture:

1. **Entry Point (`src/gen_stub.rs`)**: TokenStream parsing and generation
2. **Intermediate Representation (`src/gen_stub/*.rs`)**: `*Info` structs and business logic
3. **Code Generation**: `ToTokens` trait implementations

**Important**: When adding new functionality:
- TokenStream manipulation stays in `gen_stub.rs`
- Business logic and intermediate representations go in `gen_stub/*.rs`
- Use `ToTokens` trait for code generation, not direct `quote!` in submodules

> [!NOTE]
> For detailed design pattern documentation, see [`docs/procedural-macro-design.md`](./docs/procedural-macro-design.md)

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

For complex type definitions (e.g., `collections.abc.Callable`, overloads, generics), you can write type information directly in Python stub syntax.

**Quick Example:**

```rust
#[gen_stub_pyfunction(python = r#"
    import collections.abc
    def fn_with_callback(callback: collections.abc.Callable[[str], typing.Any]) -> None: ...
"#)]
#[pyfunction]
pub fn fn_with_callback(callback: Bound<'_, PyAny>) -> PyResult<()> { /* ... */ }
```

**Three Approaches:**
1. **Inline Python Parameter**: `#[gen_stub_pyfunction(python = "...")]` - Best for single functions
2. **Function Stub Macro**: `gen_function_from_python!` - Best for overloads
3. **Methods Stub Macro**: `gen_methods_from_python!` - Best for class method overloads

> [!NOTE]
> For detailed documentation, examples, and implementation details, see [`docs/python-stub-syntax.md`](./docs/python-stub-syntax.md)

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

The project provides automatic mappings between Rust and Python types through the `PyStubType` trait:
- Built-in types (int, str, bool, etc.)
- Collection types (Vec → list, HashMap → dict, etc.)
- PyO3 types (Bound, Py, PyAny, etc.)
- NumPy array types (feature-gated)
- Third-party crate support (either, rust_decimal)

Manual type specification is supported for edge cases where automatic translation is insufficient.

> [!NOTE]
> For detailed type mapping documentation, see [`docs/type-system.md`](./docs/type-system.md)

## Important Notes

- Python 3.10+ is the minimum supported version (do not enable 3.9 or older in PyO3 settings)
- When you think you are done with your change, use `task stub-gen` to update stub files and inspect the content
- The project uses the `inventory` crate for compile-time metadata collection across proc-macros
- Stub files are automatically included in wheel packages by maturin

## Developer Documentation

For in-depth design documentation and implementation details, see the `docs/` directory:

- [`docs/architecture.md`](./docs/architecture.md) - Overall system architecture and three-phase generation process
- [`docs/procedural-macro-design.md`](./docs/procedural-macro-design.md) - Three-layer architecture pattern for proc-macros
- [`docs/type-system.md`](./docs/type-system.md) - Rust to Python type mappings and `PyStubType` trait
- [`docs/python-stub-syntax.md`](./docs/python-stub-syntax.md) - Using Python stub syntax directly
- [`docs/default-value-arguments.md`](./docs/default-value-arguments.md) - Function parameter default values
- [`docs/default-value-members.md`](./docs/default-value-members.md) - Class attribute default values