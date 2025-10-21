# Architecture

## Overview

pyo3-stub-gen is a Python stub file (*.pyi) generator for PyO3-based Rust projects. It automatically generates Python type hints for Rust code exposed to Python through PyO3, enabling better IDE support and type checking for mixed Rust/Python projects.

## Workspace Structure

The project is organized as a Cargo workspace with the following main components:

```
pyo3-stub-gen/
├── pyo3-stub-gen/          # Core library for stub generation
├── pyo3-stub-gen-derive/   # Procedural macros for metadata collection
└── examples/               # Example projects
    ├── pure/              # Pure Rust project with Python bindings
    ├── mixed/             # Mixed Rust/Python project
    ├── mixed_sub/         # Mixed project with submodules
    └── ...                # Other maturin layout examples
```

### Component Responsibilities

- **pyo3-stub-gen**: Runtime library that generates `.pyi` files from collected metadata
- **pyo3-stub-gen-derive**: Compile-time procedural macros that extract type information from Rust code
- **examples**: Demonstration projects showing different maturin project layouts and features

## Three-Phase Generation Process

The stub generation process operates in three distinct phases:

```
┌─────────────────────────────────────────────────────────────────┐
│ Phase 1: Compile-time (pyo3-stub-gen-derive)                   │
│                                                                 │
│  Procedural macros extract type information:                   │
│  - #[gen_stub_pyclass]    → Extract class metadata             │
│  - #[gen_stub_pyfunction] → Extract function signatures        │
│  - #[gen_stub_pymethods]  → Extract method information         │
│                                                                 │
│  Uses the `inventory` crate to collect metadata across         │
│  compilation units without requiring manual registration.      │
└─────────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────────┐
│ Phase 2: Runtime Metadata Collection                           │
│                                                                 │
│  define_stub_info_gatherer! macro:                             │
│  - Gathers all metadata registered via `inventory`             │
│  - Reads pyproject.toml configuration                          │
│  - Organizes type information into module structure            │
│                                                                 │
│  Executed by running the stub_gen binary.                      │
└─────────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────────┐
│ Phase 3: Stub File Generation (pyo3-stub-gen)                  │
│                                                                 │
│  Transforms metadata into Python stub syntax:                  │
│  - Convert Rust types to Python type annotations               │
│  - Generate function/method signatures                         │
│  - Create class definitions with properties                    │
│  - Write .pyi files to disk                                    │
└─────────────────────────────────────────────────────────────────┘
```

### Phase 1: Compile-time Type Extraction

Procedural macros analyze Rust code during compilation:

```rust
#[gen_stub_pyclass]
#[pyclass]
pub struct Calculator {
    value: f64,
}

#[gen_stub_pymethods]
#[pymethods]
impl Calculator {
    #[new]
    fn new() -> Self {
        Self { value: 0.0 }
    }

    fn add(&mut self, x: f64) -> f64 {
        self.value += x;
        self.value
    }
}
```

The macros extract:
- Class names and structure
- Method signatures (names, parameters, return types)
- Documentation strings
- Type annotations
- Default values (if specified)

All extracted metadata is registered with the `inventory` crate, allowing automatic collection without manual registration in a central location.

### Phase 2: Runtime Metadata Collection

Each crate defines a stub info gatherer:

```rust
use pyo3_stub_gen::define_stub_info_gatherer;

define_stub_info_gatherer!(stub_info);
```

The `stub_info` function:
1. Collects all metadata registered via `inventory` during compilation
2. Reads configuration from `pyproject.toml` (module name, output directory, etc.)
3. Organizes type information according to the module structure
4. Returns a `StubInfo` object containing all collected data

This is typically invoked from a `src/bin/stub_gen.rs` binary:

```rust
fn main() -> Result<()> {
    let stub = my_crate::stub_info()?;
    stub.generate()?;
    Ok(())
}
```

### Phase 3: Stub File Generation

The core library transforms the collected metadata into Python stub files:

1. **Type Conversion**: Rust types → Python type annotations
   - `i32` → `int`
   - `String` → `str`
   - `Vec<T>` → `list[T]`
   - Custom PyO3 classes → their Python names

2. **Signature Generation**: Function and method signatures with type hints
   ```python
   def add(self, x: float) -> float: ...
   ```

3. **Class Definition**: Python class stubs with properties and methods
   ```python
   class Calculator:
       def __init__(self) -> None: ...
       def add(self, x: float) -> float: ...
   ```

4. **File Writing**: Organized `.pyi` files matching the Python module structure
   ```
   my_package/
   ├── __init__.pyi
   ├── submodule.pyi
   └── py.typed
   ```

## Key Design Principles

### 1. Zero Runtime Overhead

Type information extraction happens entirely at compile-time. The generated Python extension has no overhead from stub generation.

### 2. Inventory-based Collection

The `inventory` crate enables distributed metadata registration:
- No central registration point needed
- Each module can independently contribute type information
- Metadata is automatically aggregated at runtime

### 3. Separation of Concerns

- **Compile-time (derive)**: Type extraction and metadata generation
- **Runtime (core)**: Metadata collection and stub file writing
- **User code**: Minimal annotations (`#[gen_stub_*]` attributes)

### 4. PyO3 Integration

The design tightly integrates with PyO3:
- Reuses PyO3 attributes (`#[pyclass]`, `#[pyfunction]`, etc.)
- Respects PyO3 signatures (`#[pyo3(signature = ...)]`)
- Compatible with all PyO3 features (properties, class attributes, etc.)

## Data Flow Diagram

```
Rust Source Code
    │
    ├─ #[gen_stub_pyclass]
    ├─ #[gen_stub_pyfunction]
    └─ #[gen_stub_pymethods]
    │
    ↓ (Procedural macros during compilation)
    │
Metadata Structs
    │
    ├─ PyClassInfo
    ├─ PyFunctionInfo
    └─ PyMethodInfo
    │
    ↓ (inventory::submit! - automatic registration)
    │
Inventory Registry
    │
    ↓ (stub_info() call at runtime)
    │
StubInfo Collection
    │
    ├─ Read pyproject.toml
    ├─ Organize module structure
    └─ Collect all registered metadata
    │
    ↓ (stub.generate() call)
    │
Python Stub Files (.pyi)
    │
    ├─ Type annotations
    ├─ Function signatures
    └─ Class definitions
```

## Module Organization

### pyo3-stub-gen-derive

```
pyo3-stub-gen-derive/src/
├── gen_stub.rs              # Entry point - TokenStream handling
├── gen_stub/
│   ├── pyclass.rs          # PyClass metadata extraction
│   ├── pyfunction.rs       # PyFunction metadata extraction
│   ├── pymethods.rs        # PyMethods metadata extraction
│   ├── member.rs           # Class member handling
│   ├── parameter.rs        # Function parameter handling
│   ├── parse_python.rs     # Python stub syntax parsing
│   └── attr.rs             # Attribute parsing utilities
└── lib.rs                  # Macro exports
```

### pyo3-stub-gen

```
pyo3-stub-gen/src/
├── type_info.rs            # Type information data structures
├── generate/
│   ├── mod.rs             # Main generation logic
│   ├── class.rs           # Class stub generation
│   ├── function.rs        # Function stub generation
│   ├── member.rs          # Property/attribute generation
│   ├── parameters.rs      # Parameter formatting
│   └── docstring.rs       # Documentation formatting
├── stub_type/
│   ├── builtins.rs        # Python builtin type mappings
│   ├── collections.rs     # Collection type mappings
│   ├── numpy.rs           # NumPy type support
│   └── pyo3.rs            # PyO3-specific types
└── lib.rs                 # Public API
```

## Integration Points

### With PyO3

- **Attribute Placement**: `#[gen_stub_*]` macros must appear before `#[py*]` macros
- **Signature Parsing**: Extracts default values from `#[pyo3(signature = ...)]`
- **Type Inference**: Uses PyO3 type information for automatic conversion

### With Maturin

- **Build Integration**: Stub generation typically runs as part of the build process
- **Package Inclusion**: Generated `.pyi` files are automatically included in wheels
- **Project Layouts**: Supports all maturin layouts (pure, mixed, mixed_sub, etc.)

### With Python Type Checkers

- **Output Format**: Generates PEP 484-compliant stub files
- **Type Syntax**: Uses modern Python type syntax (3.10+ with `|` for unions)
- **Validation**: Stubs are tested with mypy, pyright, and stubtest

## Configuration

Configuration is read from `pyproject.toml`:

```toml
[tool.pyo3-stub-gen]
module = "my_package"
output-dir = "."
```

This allows users to control:
- Module name for generated stubs
- Output directory for `.pyi` files
- (Future) Additional generation options

## Error Handling

The system employs defensive error handling:

1. **Compile-time**: Procedural macros emit compilation errors for invalid syntax
2. **Runtime**: Stub generation returns `Result<()>` for file I/O errors
3. **Fallback**: Unsupported types fall back to `typing.Any` or `...`

## Extension Points

The architecture supports extension through:

1. **Custom Type Mappings**: Via `#[gen_stub(type = "...")]` attribute
2. **Python Stub Syntax**: Direct Python type annotation support
3. **Type Override**: Manual type specification for edge cases

## Related Documentation

- [Procedural Macro Design Pattern](./procedural-macro-design.md) - Detailed design of the derive crate
- [Type System](./type-system.md) - Rust to Python type mappings
- [Python Stub Syntax Support](./python-stub-syntax.md) - Using Python type annotations directly
- [Default Value for Function Arguments](./default-value-arguments.md) - Function parameter defaults
- [Default Value for Class Members](./default-value-members.md) - Class attribute defaults
