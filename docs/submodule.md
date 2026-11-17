# Submodule Representation in Stub Files

## Overview

This document describes the new specification for representing PyO3 submodules in Python stub files. This specification addresses [issue #351](https://github.com/Jij-Inc/pyo3-stub-gen/issues/351) and simplifies the stub generation strategy by:

1. **Protocol-based submodule representation**: Submodules created via `PyModule::add_submodule` are represented as typed attributes within the parent module's stub file, not as separate importable modules
2. **Single stub file generation**: **One `.so` file → one `.pyi` file**. All stub information is consolidated into a single `.pyi` file that corresponds to the shared library, eliminating the multi-file generation mechanism
3. **Alignment with runtime behavior**: The generated stubs accurately reflect PyO3's runtime module structure, where submodules are attributes rather than independently importable modules

## Problem Statement

### Current Issues with Multi-File Stub Generation

The current implementation generates separate stub files for each submodule (e.g., `main_mod/__init__.pyi`, `main_mod/sub_a/__init__.pyi`, `main_mod/sub_b/__init__.pyi`). This approach has several problems:

1. **Misleading import semantics**: Separate stub files suggest that `import main_mod.sub_a` should work, but PyO3's `add_submodule` does **not** make submodules independently importable
   ```python
   # These work:
   from main_mod import sub_a
   import main_mod; main_mod.sub_a

   # This FAILS at runtime with ImportError:
   import main_mod.sub_a  # 'main_mod' is not a package
   ```

2. **Misalignment with maturin**: maturin compiles Rust code into a **single shared library** (e.g., `main_mod.cpython-313-darwin.so`). The multi-file stub structure doesn't reflect this reality

3. **Complexity**: The current system maintains separate logic for pure vs. mixed layouts, tracks submodule hierarchies, and generates multiple files - all unnecessarily complex for representing a single shared library

4. **Stubtest incompatibility**: mypy's stubtest tool cannot validate nested submodules because PyO3 uses runtime attribute access rather than standard Python package structure

### PyO3 Submodule Runtime Behavior

When you use `PyModule::add_submodule`:

```rust
let sub_mod = PyModule::new(py, "sub_mod")?;
sub_mod.add_function(wrap_pyfunction!(foo, &sub_mod)?)?;
m.add_submodule(&sub_mod)?;
```

PyO3 creates a module object as an **attribute** of the parent module, not as a standalone importable module. This is fundamentally different from Python's standard package/submodule mechanism.

## New Approach: Protocol-Based Representation

### Core Principle

**Submodules are attributes, not importable modules.** The stub file should represent them as such.

### Representation Strategy

Instead of generating separate stub files, represent submodules using one of these approaches:

#### 1. Protocol-Based (Recommended)

Use Python's `typing.Protocol` to define the interface of submodule objects:

```python
import typing

# When you have multiple submodules, each gets a unique protocol name
@typing.final
class _SubMod1Protocol(typing.Protocol):
    """Type protocol for the sub_mod1 submodule."""
    def foo(self, x: int) -> int: ...
    def bar(self, name: str) -> None: ...

@typing.final
class _SubMod2Protocol(typing.Protocol):
    """Type protocol for the sub_mod2 submodule."""
    def baz(self, value: float) -> float: ...

sub_mod1: _SubMod1Protocol
sub_mod2: _SubMod2Protocol
```

**Advantages:**
- Clear that `sub_mod1` and `sub_mod2` are attributes, not importable modules
- Each submodule has a unique protocol name (no naming conflicts)
- IDE autocomplete works correctly
- Type checkers validate usage properly
- Accurately reflects runtime behavior

#### 2. Class-Based (Alternative)

Use a regular class with `@typing.final`:

```python
import typing

@typing.final
class _SubMod1:
    """The sub_mod1 submodule."""
    def foo(self, x: int) -> int: ...
    def bar(self, name: str) -> None: ...

@typing.final
class _SubMod2:
    """The sub_mod2 submodule."""
    def baz(self, value: float) -> float: ...

sub_mod1: _SubMod1
sub_mod2: _SubMod2
```

**Advantages:**
- Simpler syntax (no `Protocol` inheritance needed)
- Works with all type checkers
- Still accurately represents attribute nature
- Each submodule has a unique class name

### Naming Convention

Protocol names are generated from submodule names to ensure uniqueness:

**Generation Rule**: `_{PascalCase(submodule_name)}Protocol`

**Examples:**

| Submodule Name | Protocol Name | Notes |
|---------------|---------------|-------|
| `sub_mod` | `_SubModProtocol` | Standard snake_case conversion |
| `sub_mod1` | `_SubMod1Protocol` | Numbers preserved |
| `sub_mod2` | `_SubMod2Protocol` | Each submodule gets unique name |
| `mod_a` | `_ModAProtocol` | Single letter capitalized |
| `mod_b` | `_ModBProtocol` | Different from `mod_a` |
| `int` | `_IntProtocol` | Single word, capitalized |
| `nested` | `_NestedProtocol` | Works for nested submodules too |

**Important**: Each submodule gets a unique protocol name to avoid naming conflicts when multiple submodules exist in the same module.

**Rationale**:
- Underscore prefix indicates the type is internal to the stub file
- PascalCase conversion follows Python naming conventions for classes
- `Protocol` suffix clearly indicates these are typing protocols
- The actual attribute name (e.g., `sub_mod1`) has no underscore or suffix

## Single Stub File Generation

### Generation Rules

**For all maturin layouts** (pure or mixed), generate **exactly one stub file** that corresponds to the generated `.so` file:

| Layout | Shared Library | Stub File Location |
|--------|----------------|-------------------|
| **Pure Rust** | `{package_name}.so` | `{package_name}.pyi` |
| **Mixed Python/Rust** | `{python-source}/{module_path}.so` | `{python-source}/{module_path}.pyi` |

**Core principle**: If maturin generates `xxx.so`, pyo3-stub-gen generates `xxx.pyi` at the same location.

**Key change**: No separate files for submodules. All submodule definitions are included in the single stub file corresponding to the shared library.

### Elimination of Multi-File Logic

The following mechanisms are **removed**:

1. ❌ Submodule file generation (e.g., `main_mod/sub_a/__init__.pyi`)
2. ❌ Submodule import statements (e.g., `from . import sub_a`)
3. ❌ Hierarchical directory creation for submodules
4. ❌ Different behavior based on submodule count

### Simplified Generation Process

1. Detect the shared library output path from maturin configuration
2. Determine stub file path: replace `.so` extension with `.pyi`
3. Generate all content (including submodule protocols) into that single `.pyi` file
4. Done

**Example:**
- Shared library: `python/mixed/main_mod.cpython-313-darwin.so`
- Stub file: `python/mixed/main_mod.pyi` (same base name, different extension)

## Implementation Strategy

### Phase 1: Metadata Collection (No Change)

Compile-time and runtime metadata collection continues to work as before:

```rust
#[gen_stub_pyfunction(module = "main_mod")]
#[pyfunction]
fn greet_main() { ... }

#[gen_stub_pyfunction(module = "main_mod.sub_a")]
#[pyfunction]
fn greet_a() { ... }
```

The `module` attribute still indicates the logical grouping, but now it determines where the function appears in the single stub file.

### Phase 2: Stub Generation (Major Change)

Instead of creating separate files, the generator:

1. **Groups items by module path**:
   - Top-level items: `main_mod` → direct module members
   - Submodule items: `main_mod.sub_a` → members of `_SubAProtocol`

2. **Generates protocol definitions**:
   ```python
   @typing.final
   class _SubAProtocol(typing.Protocol):
       def greet_a() -> None: ...
   ```

3. **Adds attribute declarations**:
   ```python
   sub_a: _SubAProtocol
   ```

4. **Writes everything to single file**: All protocols and attributes in one `.pyi` file

### Data Structure Changes

```rust
// Before: separate ModuleInfo for each module
// Each module generates its own file

// After: single ModuleInfo with nested submodule information
pub struct ModuleInfo {
    pub name: String,
    pub items: Vec<StubItem>,
    pub submodules: HashMap<String, SubmoduleInfo>,  // NEW
}

pub struct SubmoduleInfo {
    pub name: String,
    pub protocol_name: String,  // e.g., "_ModAProtocol", "_SubMod1Protocol"
    pub items: Vec<StubItem>,
    pub submodules: HashMap<String, SubmoduleInfo>,  // Recursive for nested submodules
}

impl SubmoduleInfo {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            protocol_name: Self::generate_protocol_name(name),
            items: vec![],
            submodules: HashMap::new(),
        }
    }

    /// Generate protocol name from submodule name
    /// e.g., "sub_mod1" -> "_SubMod1Protocol"
    fn generate_protocol_name(submodule_name: &str) -> String {
        let pascal_case = to_pascal_case(submodule_name);
        format!("_{}Protocol", pascal_case)
    }
}
```

**Note**: `SubmoduleInfo` is recursive to support nested submodules (e.g., `module.sub.nested`). The `generate_protocol_name` function ensures each submodule has a unique protocol name.

### Code Generation Changes

```rust
// Generate protocol for each submodule
// Each submodule gets a unique protocol name (e.g., _SubMod1Protocol, _SubMod2Protocol)
for (submod_name, submod_info) in module.submodules {
    writeln!(f, "@typing.final")?;
    writeln!(f, "class {}(typing.Protocol):", submod_info.protocol_name)?;
    writeln!(f, "    \"\"\"Type protocol for the {} submodule.\"\"\"", submod_name)?;

    for item in &submod_info.items {
        write_stub_item(f, item, indent = 1)?;
    }

    writeln!(f)?;
}

// Generate attribute declarations
// Each submodule attribute is typed with its unique protocol
for (submod_name, submod_info) in module.submodules {
    writeln!(f, "{}: {}", submod_name, submod_info.protocol_name)?;
}
```

**Example output for two submodules:**
```python
@typing.final
class _SubMod1Protocol(typing.Protocol):
    """Type protocol for the sub_mod1 submodule."""
    def func1() -> None: ...

@typing.final
class _SubMod2Protocol(typing.Protocol):
    """Type protocol for the sub_mod2 submodule."""
    def func2() -> None: ...

sub_mod1: _SubMod1Protocol
sub_mod2: _SubMod2Protocol
```

## Examples

### Pure Rust Layout

**Project Structure:**
```
pure/
├── src/
│   └── lib.rs
├── pyproject.toml
└── pure.pyi          # Single generated stub
```

**Rust Code:**
```rust
#[gen_stub_pyfunction(module = "pure")]
#[pyfunction]
fn greet_main() -> PyResult<String> { ... }

#[gen_stub_pyfunction(module = "pure.sub_a")]
#[pyfunction]
fn greet_a() -> PyResult<String> { ... }

#[gen_stub_pyfunction(module = "pure.sub_b")]
#[pyfunction]
fn greet_b() -> PyResult<String> { ... }

#[pymodule]
fn pure(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(greet_main, m)?)?;

    let sub_a = PyModule::new(m.py(), "sub_a")?;
    sub_a.add_function(wrap_pyfunction!(greet_a, &sub_a)?)?;
    m.add_submodule(&sub_a)?;

    let sub_b = PyModule::new(m.py(), "sub_b")?;
    sub_b.add_function(wrap_pyfunction!(greet_b, &sub_b)?)?;
    m.add_submodule(&sub_b)?;

    Ok(())
}
```

**Generated `pure.pyi`:**
```python
# This file is automatically generated by pyo3_stub_gen
# ruff: noqa: E501, F401

import typing

# Submodule protocols
@typing.final
class _SubAProtocol(typing.Protocol):
    """Type protocol for the sub_a submodule."""
    def greet_a() -> str: ...

@typing.final
class _SubBProtocol(typing.Protocol):
    """Type protocol for the sub_b submodule."""
    def greet_b() -> str: ...

# Module members
def greet_main() -> str: ...

# Submodule attributes
sub_a: _SubAProtocol
sub_b: _SubBProtocol
```

**Python Usage:**
```python
import pure

# All of these work correctly:
result1 = pure.greet_main()
result2 = pure.sub_a.greet_a()
result3 = pure.sub_b.greet_b()

from pure import sub_a
result4 = sub_a.greet_a()

# This correctly fails (both at runtime AND type-checking):
import pure.sub_a  # Error: 'pure' is not a package
```

### Mixed Python/Rust Layout

**Project Structure:**
```
mixed/
├── python/
│   └── mixed/
│       ├── main_mod.cpython-313-darwin.so
│       └── main_mod.pyi        # Single generated stub (same name as .so)
├── src/
│   └── lib.rs
└── pyproject.toml
```

**pyproject.toml:**
```toml
[tool.maturin]
python-source = "python"
module-name = "mixed.main_mod"
```

**Rust Code:**
```rust
#[gen_stub_pyfunction(module = "mixed.main_mod")]
#[pyfunction]
fn greet_main() -> PyResult<String> { ... }

#[gen_stub_pyfunction(module = "mixed.main_mod.mod_a")]
#[pyfunction]
fn greet_a() -> PyResult<String> { ... }

#[gen_stub_pyfunction(module = "mixed.main_mod.mod_b")]
#[pyfunction]
fn greet_b() -> PyResult<String> { ... }

#[pymodule]
fn main_mod(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(greet_main, m)?)?;

    let mod_a = PyModule::new(m.py(), "mod_a")?;
    mod_a.add_function(wrap_pyfunction!(greet_a, &mod_a)?)?;
    m.add_submodule(&mod_a)?;

    let mod_b = PyModule::new(m.py(), "mod_b")?;
    mod_b.add_function(wrap_pyfunction!(greet_b, &mod_b)?)?;
    m.add_submodule(&mod_b)?;

    Ok(())
}
```

**Generated `python/mixed/main_mod.pyi`:**
```python
# This file is automatically generated by pyo3_stub_gen
# ruff: noqa: E501, F401

import typing

# Submodule protocols
@typing.final
class _ModAProtocol(typing.Protocol):
    """Type protocol for the mod_a submodule."""
    def greet_a() -> str: ...

@typing.final
class _ModBProtocol(typing.Protocol):
    """Type protocol for the mod_b submodule."""
    def greet_b() -> str: ...

# Module members
def greet_main() -> str: ...

# Submodule attributes
mod_a: _ModAProtocol
mod_b: _ModBProtocol
```

**Python Usage:**
```python
from mixed import main_mod

# These work:
result1 = main_mod.greet_main()
result2 = main_mod.mod_a.greet_a()
result3 = main_mod.mod_b.greet_b()

from mixed.main_mod import mod_a, mod_b
result4 = mod_a.greet_a()
result5 = mod_b.greet_b()

# This correctly fails:
import mixed.main_mod.mod_a  # Error: cannot import name 'mod_a' from 'mixed.main_mod'
```

### Nested Submodules

**Rust Code:**
```rust
#[gen_stub_pyfunction(module = "mymod")]
#[pyfunction]
fn root_fn() { ... }

#[gen_stub_pyfunction(module = "mymod.sub")]
#[pyfunction]
fn sub_fn() { ... }

#[gen_stub_pyfunction(module = "mymod.sub.nested")]
#[pyfunction]
fn nested_fn() { ... }

#[pymodule]
fn mymod(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(root_fn, m)?)?;

    let sub = PyModule::new(m.py(), "sub")?;
    sub.add_function(wrap_pyfunction!(sub_fn, &sub)?)?;

    let nested = PyModule::new(m.py(), "nested")?;
    nested.add_function(wrap_pyfunction!(nested_fn, &nested)?)?;
    sub.add_submodule(&nested)?;

    m.add_submodule(&sub)?;
    Ok(())
}
```

**Generated `mymod.pyi`:**
```python
import typing

# Nested submodule protocols
@typing.final
class _NestedProtocol(typing.Protocol):
    """Type protocol for the nested submodule."""
    def nested_fn() -> None: ...

@typing.final
class _SubProtocol(typing.Protocol):
    """Type protocol for the sub submodule."""
    def sub_fn() -> None: ...
    nested: _NestedProtocol

# Module members
def root_fn() -> None: ...

# Submodule attributes
sub: _SubProtocol
```

**Python Usage:**
```python
import mymod

mymod.root_fn()
mymod.sub.sub_fn()
mymod.sub.nested.nested_fn()  # Nested access works

from mymod import sub
sub.sub_fn()
sub.nested.nested_fn()
```

## Migration Guide

### For Library Users

**Before (old multi-file approach):**
```python
# This appeared to work in type checking but failed at runtime
import mymodule.submodule  # Type checker: OK, Runtime: ERROR
```

**After (new protocol approach):**
```python
# Type checker and runtime agree
from mymodule import submodule  # Both: OK
import mymodule; mymodule.submodule  # Both: OK
import mymodule.submodule  # Both: ERROR
```

**What changes for users:**
- Type checking now accurately reflects runtime behavior
- Invalid import patterns are caught at type-check time
- No functional changes to working code

### For Library Developers

**Changes needed:**

1. **Remove layout-specific configuration workarounds**: No need for mixed layout just to get multiple stub files
   ```toml
   # Before: Had to use mixed layout for submodules
   [tool.maturin]
   python-source = "python"  # Just for stub files

   # After: Pure layout works fine
   [tool.maturin]
   # No python-source needed
   ```

2. **Regenerate stubs**: Run `cargo run --bin stub_gen` to generate new protocol-based stubs

3. **Update tests**: If tests validated multi-file stub structure, update them

4. **Documentation**: Update any documentation that referenced separate submodule stub files

**What stays the same:**
- Rust code structure (no changes to `#[pymodule]`, `add_submodule`, etc.)
- Proc-macro attributes (`#[gen_stub_pyfunction(module = "...")]`)
- Runtime Python API (all the same imports still work)

## Benefits

### 1. Accurate Runtime Representation

The stub files now accurately reflect PyO3's actual behavior:
- Submodules are attributes ✅
- Cannot `import module.submodule` ✅
- Can `from module import submodule` ✅

### 2. Simplified Implementation

- **1:1 correspondence**: Each `.so` file has exactly one `.pyi` file
- Single file generation logic (no multi-file complexity)
- No layout-specific submodule handling
- No directory hierarchy management
- Easier to test and maintain

### 3. Better Type Checker Compatibility

- Type checkers can validate the actual usage patterns
- No false positives for invalid import styles
- Protocol-based types work across all type checkers (mypy, pyright, pytype)

### 4. Stubtest Compatibility

- Single stub file matches single shared library
- No nested submodule issues
- Stubtest can validate the entire public API

### 5. Layout Independence

- Pure Rust layout fully supports submodules
- No need for mixed layout workarounds
- Consistent behavior across all project types

## Related Issues and PRs

- [Issue #351](https://github.com/Jij-Inc/pyo3-stub-gen/issues/351) - Original issue describing the problem
- `docs/stub-file-generation.md` - Previous multi-file generation documentation (to be replaced)

## See Also

- [Architecture](./architecture.md) - Overall system architecture
- [Type System](./type-system.md) - Rust to Python type mappings
- [Python Stub Syntax](./python-stub-syntax.md) - Manual stub syntax specification
