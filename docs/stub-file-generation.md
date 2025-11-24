# Stub File Generation Rules

## Overview

This document describes the rules and behavior of stub file (`.pyi`) generation in pyo3-stub-gen, including:
- How the generator detects pure Rust vs. mixed Python/Rust projects
- How it decides between creating single-file stubs (`module.pyi`) versus directory-based stubs (`module/__init__.pyi`)
- maturin's constraints on stub file packaging and pyo3-stub-gen's design philosophy

### Design Philosophy: Alignment with maturin

**Important**: pyo3-stub-gen's stub file generation is designed to work with [maturin](https://github.com/PyO3/maturin), the recommended build tool for PyO3 projects. The generator follows maturin's stub file packaging constraints:

- **Pure Rust layout**: maturin accepts **only a single stub file** at the module root level
  - Example: `mymodule.pyi` for a module named `mymodule`
  - **Limitation**: Submodules within the shared library are **not supported** by maturin in pure Rust layout
  - If you need submodules, you **must** use mixed Python/Rust layout

- **Mixed Python/Rust layout**: maturin supports full stub file hierarchies
  - Example: `python/mypackage/mymodule/__init__.pyi`, `python/mypackage/mymodule/sub.pyi`
  - **Recommended** for projects with submodules created via PyO3's `add_submodule`

**pyo3-stub-gen's stance**: We follow maturin's constraints and recommend using mixed Python/Rust layout for any project that requires submodules, even if there is no actual Python source code. This ensures generated stub files can be properly packaged and distributed.

## Layout Detection: Pure Rust vs. Mixed Python/Rust

The generator automatically detects the project layout by reading `pyproject.toml` configuration.

### Detection Logic

**Implementation**: `pyo3-stub-gen/src/pyproject.rs:48-62` and `pyo3-stub-gen/src/generate/stub_info.rs:78-90`

The detection process:
1. `PyProject::python_source()` returns `Some(path)` if `[tool.maturin]` has `python-source`, otherwise `None`
2. `StubInfoBuilder::from_pyproject_toml()` sets:
   - `is_mixed_layout = python_source().is_some()`
   - `python_root` to the `python-source` path if present, otherwise `CARGO_MANIFEST_DIR`

### Layout Determination Rules

| Layout | `pyproject.toml` Configuration | `python_root` Value | Submodule Support |
|--------|-------------------------------|---------------------|-------------------|
| **Pure Rust** | No `[tool.maturin]` section or no `python-source` key | `CARGO_MANIFEST_DIR` (project root) | ❌ Not supported by maturin |
| **Mixed Python/Rust** | Has `[tool.maturin]` with `python-source = "path"` | Specified path (e.g., `"python"`) | ✅ Fully supported |

**Examples:**

**Pure Rust** (`examples/pure/pyproject.toml`):
```toml
[tool.maturin]
features = ["pyo3/extension-module"]
# No python-source specified
```
→ `python_root` = project root (where `pyproject.toml` is located)

**Mixed Python/Rust** (`examples/mixed/pyproject.toml`):
```toml
[tool.maturin]
python-source = "python"
module-name = "mixed.main_mod"
```
→ `python_root` = `<project_root>/python`

**Key Points**:
- The presence or absence of `python-source` in `[tool.maturin]` is the **sole determinant** of layout type
- **Pure Rust layout limitations**: maturin will **only package a single stub file** (e.g., `mymodule.pyi`). If you generate submodule stubs (e.g., `mymodule/__init__.pyi`, `mymodule/sub.pyi`), they will **not be included** in the wheel package
- **For submodules**: Always use mixed Python/Rust layout, even if you have no Python source files

## Stub File Generation Logic

The stub file generation logic is implemented in `pyo3-stub-gen/src/generate/stub_info.rs`. The output path is determined **solely by the layout type** detected from `pyproject.toml`:

### Simplified Generation Rules

| Layout Type | Output Path | Format |
|-------------|-------------|---------|
| **Pure Rust** | `$CARGO_MANIFEST_DIR/{package_name}.pyi` | Single file |
| **Mixed Python/Rust** | `{python-source}/{module_path}/__init__.pyi` | Directory-based (all modules) |

**Key simplification**: The layout type (presence/absence of `python-source`) **entirely determines** the stub file format:

- **Pure Rust**: Always generates a single `.pyi` file at the project root (e.g., `mymodule.pyi`)
- **Mixed Python/Rust**: Always generates `__init__.pyi` files in directories for **every module** (including submodules)
  - Top-level module: `{python-source}/mypackage/mymodule/__init__.pyi`
  - Submodules: `{python-source}/mypackage/mymodule/sub_a/__init__.pyi`, etc.

There is no conditional logic based on submodules or directory existence.

### Rationale

This simplified approach:

1. **Aligns with maturin's constraints**:
   - Pure Rust: maturin only packages single stub files → always generate single file
   - Mixed: maturin supports full hierarchies → always use directory structure

2. **Eliminates ambiguity**: No need to check for pre-existing directories or count submodules

3. **Enforces best practices**: Projects with submodules must use mixed layout (where maturin properly packages them)

4. **Consistent behavior**: Same layout type always produces same file structure

### Path Normalization

Before determining the output path, the module name undergoes normalization:

1. **Dash to underscore conversion**: Package names with dashes are converted to underscores for Python compatibility
   ```rust
   let normalized_name = name.replace("-", "_");
   ```

2. **Dot to slash conversion**: Module path separators are converted to filesystem separators
   ```rust
   let path = normalized_name.replace(".", "/");
   ```

Example: `my-package.sub_module` → `my_package/sub_module`

## Submodule Detection and Import Generation

Submodules are automatically detected during the build phase to generate proper import statements in stub files. **Note**: This detection does **not** affect the stub file path decision (which is determined solely by layout type), but it controls the content of `__init__.pyi` files.

**Detection Logic**: `pyo3-stub-gen/src/generate/stub_info.rs:113-141`

The `register_submodules` method automatically detects module hierarchies and creates missing parent modules:

1. Parses module names by splitting on `.` to identify parent-child relationships
2. Groups children by their parent modules
3. Creates empty parent modules as needed (e.g., if only `package.main_mod.sub_a` is defined, it creates `package` and `package.main_mod`)
4. Registers submodule imports in each parent's `__init__.pyi`

**Key feature**: Automatically synthesizes intermediate parent modules even when only leaf modules are explicitly defined via `#[gen_stub_pyfunction(module = "...")]`.

**Purpose**: In mixed Python/Rust layout, detected submodules are added as import statements in the parent's `__init__.pyi`:

```python
# mixed/main_mod/__init__.pyi
from . import mod_a
from . import mod_b
from . import int
```

**Example:**

Given these modules registered via `#[gen_stub_pyfunction(module = "...")]`:
- `mixed.main_mod`
- `mixed.main_mod.mod_a`
- `mixed.main_mod.mod_b`
- `mixed.main_mod.int`

The detector creates:
- `mixed.main_mod.submodules = {"mod_a", "mod_b", "int"}`

This results in:
- **Files generated**: `main_mod/__init__.pyi`, `main_mod/mod_a/__init__.pyi`, `main_mod/mod_b/__init__.pyi`, `main_mod/int/__init__.pyi`
- **Imports in `__init__.pyi`**: `from . import mod_a`, etc.

## Maturin Layout Examples

### Pure Layout

**Project Structure:**
```
pure/
├── src/
│   └── lib.rs
├── pyproject.toml
└── pure.pyi          # Generated stub
```

**pyproject.toml:**
```toml
[tool.maturin]
# No python-source specified
```

**Layout Detection:**
- No `python-source` in `pyproject.toml`
- **Detected as**: Pure Rust layout
- **`python_root`**: `CARGO_MANIFEST_DIR` (project root)

**Behavior:**
- Module name: `pure` (from package name)
- No submodules: All code in single module
- No pre-existing directory
- **Output**: `pure.pyi` (single file at project root)

**⚠️ Important**: This layout **only supports a single stub file**. If your Rust code defines submodules (e.g., using `#[gen_stub_pyfunction(module = "pure.sub")]`), pyo3-stub-gen will generate multiple stub files, but **maturin will only package `pure.pyi` and ignore the rest**. Use mixed layout instead if you need submodules.

### Mixed Layout

**Project Structure:**
```
mixed/
├── python/
│   └── mixed/
│       ├── main_mod.cpython-313-darwin.so
│       └── main_mod/                  # Directory for stubs
│           ├── __init__.pyi           # Generated stub
│           ├── mod_a/
│           │   └── __init__.pyi       # Generated stub
│           ├── mod_b/
│           │   └── __init__.pyi       # Generated stub
│           └── int/
│               └── __init__.pyi       # Generated stub
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
// Main module
#[gen_stub_pyfunction(module = "mixed.main_mod")]
#[pyfunction]
fn greet_main() { ... }

// Submodule A
#[gen_stub_pyfunction(module = "mixed.main_mod.mod_a")]
#[pyfunction]
fn greet_a() { ... }

// Submodule B
#[gen_stub_pyfunction(module = "mixed.main_mod.mod_b")]
#[pyfunction]
fn greet_b() { ... }

// Submodule int
#[gen_stub_pyfunction(module = "mixed.main_mod.int")]
#[pyfunction]
fn dummy_int_fun(x: usize) -> usize { ... }

#[pymodule]
fn main_mod(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(greet_main, m)?)?;

    // Add submodules
    let mod_a = PyModule::new(py, "mod_a")?;
    mod_a.add_function(wrap_pyfunction!(greet_a, &mod_a)?)?;
    m.add_submodule(&mod_a)?;

    // ... similar for mod_b and int
    Ok(())
}
```

**Layout Detection:**
- Has `python-source = "python"` in `pyproject.toml`
- **Detected as**: Mixed Python/Rust layout
- **`python_root`**: `<project_root>/python`

**Behavior:**
- Module name: `mixed.main_mod`
- **Has submodules**: `mod_a`, `mod_b`, `int` detected via module paths
- **Output**:
  - `python/mixed/main_mod/__init__.pyi` (main module)
  - `python/mixed/main_mod/mod_a/__init__.pyi` (submodule)
  - `python/mixed/main_mod/mod_b/__init__.pyi` (submodule)
  - `python/mixed/main_mod/int/__init__.pyi` (submodule)

**Coexistence:**
- `main_mod.cpython-313-darwin.so` (native module **file**)
- `main_mod/` (stub **directory**)
- Both coexist in `python/mixed/` thanks to Python's import system

**Generated `__init__.pyi`:**
```python
# This file is automatically generated by pyo3_stub_gen
# ruff: noqa: E501, F401

import builtins
import typing
from . import int      # Submodule reference
from . import mod_a    # Submodule reference
from . import mod_b    # Submodule reference

@typing.final
class A:
    def show_x(self) -> None: ...

def greet_main() -> None: ...
```

## Implementation Details

### Layout Detection Implementation

**Location**: `pyo3-stub-gen/src/pyproject.rs` and `pyo3-stub-gen/src/generate/stub_info.rs`

The layout detection happens in three stages:

1. **Parse `pyproject.toml`** (`pyproject.rs:27-35`): Reads and validates the TOML file
2. **Extract `python-source`** (`pyproject.rs:48-62`): Returns `Some(path)` if `[tool.maturin]` has `python-source`, otherwise `None`
3. **Determine layout** (`stub_info.rs:78-90`): Sets `is_mixed_layout` and `python_root` based on `python_source()` result

**Result:**
- Mixed layout: `is_mixed_layout = true`, `python_root = <project>/python-source`
- Pure layout: `is_mixed_layout = false`, `python_root = CARGO_MANIFEST_DIR`

### File Creation Process

**Location**: `pyo3-stub-gen/src/generate/stub_info.rs:38-67`

The `StubInfo::generate()` method:

1. **Path normalization**: Converts dashes to underscores and dots to slashes
2. **Layout-based decision**:
   - **Pure Rust**: Generates single file `{package_name}.pyi` at project root
   - **Mixed Python/Rust**: Generates `{module_path}/__init__.pyi` for all modules
3. **Directory creation**: Automatically creates parent directories as needed

## Related PRs and Changes

### Simplified Stub Generation Strategy

**Evolution**: The current approach simplifies stub generation by depending **solely on layout type**, superseding PR #348's directory existence checking approach.

**Key change**: Instead of conditional logic based on runtime state (submodules, directory existence), the generator now uses:
- **Pure Rust layout** → Always single file
- **Mixed Python/Rust layout** → Always directory-based `__init__.pyi`

**Benefits:**
1. Aligns with maturin's packaging constraints
2. Eliminates runtime state dependencies
3. Predictable behavior
4. Enforces best practices (submodules require mixed layout)

## Best Practices

### Choosing the Right Layout

1. **Pure Rust layout**: Use when your project meets **all** of these conditions:
   - No Python source code
   - **No submodules** (no use of PyO3's `add_submodule`)
   - Single module at the top level only

   **Configuration:**
   - Omit `python-source` from `[tool.maturin]`
   - Stub files are generated at the project root
   - Simplest setup for pure PyO3 projects

   **⚠️ Limitation**: maturin will **only package one stub file** (e.g., `mymodule.pyi`). Directory-based stubs are **ignored** in pure Rust layout.

2. **Mixed Python/Rust layout**: Use when **any** of these apply:
   - You have Python source files alongside Rust code
   - **You need submodules** (using PyO3's `add_submodule`)
   - You want to organize Rust modules with dot notation (e.g., `mypackage.main_mod.sub_a`)

   **Configuration:**
   - Add `python-source = "python"` to `[tool.maturin]`
   - Organize Python files under the specified directory (if any)
   - Stub files are generated in the same directory as Python sources
   - Enables seamless integration of Python and Rust modules

   **✅ Recommended**: Even if you have no Python source code, use this layout if you need submodules. This ensures all generated stub files are properly packaged by maturin.

### Code Organization

3. **For projects with submodules**: Always use mixed Python/Rust layout
   ```toml
   # pyproject.toml
   [tool.maturin]
   python-source = "python"
   module-name = "mypackage.main_mod"
   ```

   Even if you have no Python source files, this configuration is **required** for maturin to package submodule stubs correctly.

4. **Use explicit module paths**: Always specify `module = "..."` in procedural macros for clarity
   ```rust
   #[gen_stub_pyfunction(module = "mypackage.mymodule")]
   #[pyfunction]
   fn my_function() { ... }
   ```

5. **Organize submodules consistently**: Use dot notation in module paths to create logical hierarchies
   ```rust
   module = "mypackage.main_mod.submodule_a"  // Good - submodule
   module = "mypackage.main_mod"              // Parent module
   ```

6. **Match Rust submodules to Python structure**: When using `add_submodule`, ensure module paths match
   ```rust
   // In your #[pymodule] function:
   let sub = PyModule::new(py, "mod_a")?;
   sub.add_function(wrap_pyfunction!(greet_a, &sub)?)?;
   m.add_submodule(&sub)?;

   // In your function definition:
   #[gen_stub_pyfunction(module = "mypackage.main_mod.mod_a")]
   #[pyfunction]
   fn greet_a() { ... }
   ```

### Layout Detection Verification

7. **Verify your layout is correctly detected**: Check where stub files are generated
   ```bash
   # Pure Rust: stubs at project root
   ls *.pyi

   # Mixed: stubs in python-source directory
   ls python/**/*.pyi
   ```

8. **If stubs are in the wrong location**: Check your `pyproject.toml` configuration
   ```toml
   [tool.maturin]
   python-source = "python"  # Add this for mixed layout
   ```

### Common Pitfalls

9. **❌ Do NOT use pure Rust layout with submodules**
    ```rust
    // This will generate incomplete stubs!
    #[gen_stub_pyfunction(module = "mymodule.sub")]
    #[pyfunction]
    fn my_function() { ... }
    ```

    **Problem**: With pure Rust layout, pyo3-stub-gen will **only generate `mymodule.pyi`** (single file). All submodule definitions will be merged into this single file, which doesn't match the runtime module structure created by PyO3's `add_submodule`.

    **Result**: Type checkers won't find `mymodule.sub` as a separate module, causing import errors.

    **Solution**: Use mixed Python/Rust layout for any project with submodules:
    ```toml
    # pyproject.toml
    [tool.maturin]
    python-source = "python"
    module-name = "mymodule"
    ```

    Then re-run stub generation:
    ```bash
    cargo run --bin stub_gen
    # Stubs will now be generated at:
    # python/mymodule/__init__.pyi
    # python/mymodule/sub/__init__.pyi
    ```

## See Also

- [Architecture](./architecture.md) - Overall system design
- [Type System](./type-system.md) - Rust to Python type mappings
- [Python Stub Syntax](./python-stub-syntax.md) - Manual stub syntax specification
