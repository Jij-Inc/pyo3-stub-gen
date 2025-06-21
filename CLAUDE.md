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

## Testing Strategy

Each example includes comprehensive testing:
- `pytest` - Python functionality tests
- `pyright` - Static type checking validation
- `ruff` - Python code linting
- Tests verify that generated stubs provide correct type information

## Type System

The project provides mappings between Rust and Python types:
- `/stub_type/builtins.rs` - Basic Python types
- `/stub_type/collections.rs` - Python collections
- `/stub_type/numpy.rs` - NumPy array types
- `/stub_type/pyo3.rs` - PyO3-specific types

Manual type specification is supported for edge cases where automatic translation fails.
