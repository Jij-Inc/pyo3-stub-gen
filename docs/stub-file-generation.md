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

**Implementation**: `pyo3-stub-gen/src/pyproject.rs:48-62`

```rust
/// Return `tool.maturin.python_source` if it exists, which means the project is a mixed Rust/Python project.
pub fn python_source(&self) -> Option<PathBuf> {
    if let Some(tool) = &self.tool {
        if let Some(maturin) = &tool.maturin {
            if let Some(python_source) = &maturin.python_source {
                if let Some(base) = self.toml_path.parent() {
                    return Some(base.join(python_source));
                } else {
                    return Some(PathBuf::from(python_source));
                }
            }
        }
    }
    None
}
```

**Usage**: `pyo3-stub-gen/src/generate/stub_info.rs:78-90`

```rust
fn from_pyproject_toml(pyproject: PyProject) -> Self {
    let is_mixed_layout = pyproject.python_source().is_some();
    let python_root = pyproject
        .python_source()  // Returns Some(path) for mixed, None for pure
        .unwrap_or(PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()));

    Self {
        modules: BTreeMap::new(),
        default_module_name: pyproject.module_name().to_string(),
        python_root,
        is_mixed_layout,
    }
}
```

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

**Detection Logic** (`register_submodules` method - `pyo3-stub-gen/src/generate/stub_info.rs:113-141`):

```rust
fn register_submodules(&mut self) {
    let mut all_parent_child_pairs: Vec<(String, String)> = Vec::new();

    // For each existing module, collect all parent-child relationships
    for module in self.modules.keys() {
        let path = module.split('.').collect::<Vec<_>>();

        // Generate all parent paths and their immediate children
        for i in 1..path.len() {
            let parent = path[..i].join(".");
            let child = path[i].to_string();
            all_parent_child_pairs.push((parent, child));
        }
    }

    // Group children by parent
    let mut parent_to_children: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (parent, child) in all_parent_child_pairs {
        parent_to_children.entry(parent).or_default().insert(child);
    }

    // Create or update all parent modules
    for (parent, children) in parent_to_children {
        let module = self.modules.entry(parent.clone()).or_default();
        module.name = parent;
        module.default_module_name = self.default_module_name.clone();
        module.submodules.extend(children);
    }
}
```

**Process:**

1. **Collect parent-child relationships**: For each registered module, split its name by `.` and generate all parent-child pairs
   - Example: `package.main_mod.sub_a` generates:
     - `package` → `main_mod`
     - `package.main_mod` → `sub_a`
2. **Group children by parent**: Use a map to collect all children for each parent module
3. **Create or update parent modules**: For each parent:
   - Create the parent module if it doesn't exist (using `.entry().or_default()`)
   - Set the module's `name` and `default_module_name`
   - Extend the `submodules` set with all children

**Key enhancement**: This implementation automatically creates missing parent modules, ensuring that even if only child modules are explicitly defined (e.g., via `#[gen_stub_pyfunction(module = "package.main_mod.sub_a")]`), all intermediate parent modules (`package`, `package.main_mod`) are synthesized with proper submodule imports.

**Purpose**: In mixed Python/Rust layout, detected submodules are added as import statements in the parent's `__init__.pyi`:

```python
# mixed_sub/main_mod/__init__.pyi
from . import mod_a
from . import mod_b
from . import int
```

**Example:**

Given these modules registered via `#[gen_stub_pyfunction(module = "...")]`:
- `mixed_sub.main_mod`
- `mixed_sub.main_mod.mod_a`
- `mixed_sub.main_mod.mod_b`
- `mixed_sub.main_mod.int`

The detector creates:
- `mixed_sub.main_mod.submodules = {"mod_a", "mod_b", "int"}`

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
│       ├── __init__.py
│       ├── main_mod.cpython-313-darwin.so
│       └── main_mod/
│           └── __init__.pyi          # Generated stub
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
#[gen_stub_pyclass]
#[pyclass(module = "mixed.main_mod")]
struct A { x: usize }

#[pymodule]
fn main_mod(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<A>()?;
    Ok(())
}
```

**Layout Detection:**
- Has `python-source = "python"` in `pyproject.toml`
- **Detected as**: Mixed Python/Rust layout
- **`python_root`**: `<project_root>/python`

**Behavior:**
- Module name: `mixed.main_mod` (explicit in maturin config)
- No submodules: All classes/functions in `mixed.main_mod`
- **Output**: `python/mixed/main_mod/__init__.pyi` (directory-based)

**Coexistence:**
- `main_mod.cpython-313-darwin.so` (native module **file**)
- `main_mod/` (stub **directory**)
- Both coexist in `python/mixed/` thanks to Python's import system

### Mixed Sub Layout

**Project Structure:**
```
mixed_sub/
├── python/
│   └── mixed_sub/
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
module-name = "mixed_sub.main_mod"
```

**Rust Code:**
```rust
// Main module
#[gen_stub_pyfunction(module = "mixed_sub.main_mod")]
#[pyfunction]
fn greet_main() { ... }

// Submodule A
#[gen_stub_pyfunction(module = "mixed_sub.main_mod.mod_a")]
#[pyfunction]
fn greet_a() { ... }

// Submodule B
#[gen_stub_pyfunction(module = "mixed_sub.main_mod.mod_b")]
#[pyfunction]
fn greet_b() { ... }

// Submodule int
#[gen_stub_pyfunction(module = "mixed_sub.main_mod.int")]
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
- Module name: `mixed_sub.main_mod`
- **Has submodules**: `mod_a`, `mod_b`, `int` detected via module paths
- **Output**:
  - `python/mixed_sub/main_mod/__init__.pyi` (main module)
  - `python/mixed_sub/main_mod/mod_a/__init__.pyi` (submodule)
  - `python/mixed_sub/main_mod/mod_b/__init__.pyi` (submodule)
  - `python/mixed_sub/main_mod/int/__init__.pyi` (submodule)

**Coexistence:**
- `main_mod.cpython-313-darwin.so` (native module **file**)
- `main_mod/` (stub **directory**)
- Both coexist in `python/mixed_sub/` thanks to Python's import system

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

The layout detection and `python_root` determination happens in two stages:

1. **Parse `pyproject.toml`** (`pyproject.rs:27-35`):
   ```rust
   pub fn parse_toml(path: impl AsRef<Path>) -> Result<Self> {
       let path = path.as_ref();
       if path.file_name() != Some("pyproject.toml".as_ref()) {
           bail!("{} is not a pyproject.toml", path.display())
       }
       let mut out: PyProject = toml::de::from_str(&fs::read_to_string(path)?)?;
       out.toml_path = path.to_path_buf();
       Ok(out)
   }
   ```

2. **Extract `python-source` if present** (`pyproject.rs:48-62`):
   ```rust
   pub fn python_source(&self) -> Option<PathBuf> {
       if let Some(tool) = &self.tool {
           if let Some(maturin) = &tool.maturin {
               if let Some(python_source) = &maturin.python_source {
                   if let Some(base) = self.toml_path.parent() {
                       return Some(base.join(python_source));
                   } else {
                       return Some(PathBuf::from(python_source));
                   }
               }
           }
       }
       None
   }
   ```

3. **Determine layout type and `python_root`** (`stub_info.rs:78-90`):
   ```rust
   fn from_pyproject_toml(pyproject: PyProject) -> Self {
       let is_mixed_layout = pyproject.python_source().is_some();
       let python_root = pyproject
           .python_source()  // Returns Some(path) for mixed, None for pure
           .unwrap_or(PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()));

       Self {
           modules: BTreeMap::new(),
           default_module_name: pyproject.module_name().to_string(),
           python_root,
           is_mixed_layout,
       }
   }
   ```

**The flow:**
- Mixed layout: `python_source()` → `Some("python")` → `is_mixed_layout = true`, `python_root = <project>/python`
- Pure layout: `python_source()` → `None` → `is_mixed_layout = false`, `python_root = CARGO_MANIFEST_DIR`

### File Creation Process (Simplified)

**Location**: `pyo3-stub-gen/src/generate/stub_info.rs:31-55`

The simplified generation logic:

```rust
pub fn generate(&self) -> Result<()> {
    for (name, module) in self.modules.iter() {
        // Convert dashes to underscores for Python compatibility
        let normalized_name = name.replace("-", "_");
        let path = normalized_name.replace(".", "/");

        // Determine destination path based solely on layout type
        let dest = if self.is_mixed_layout {
            // Mixed Python/Rust: Always use directory-based structure
            self.python_root.join(&path).join("__init__.pyi")
        } else {
            // Pure Rust: Always use single file at root (use first segment of module name)
            let package_name = normalized_name.split('.').next().unwrap();
            self.python_root.join(format!("{package_name}.pyi"))
        };

        let dir = dest.parent().context("Cannot get parent directory")?;
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }

        let mut f = fs::File::create(&dest)?;
        write!(f, "{module}")?;
        log::info!(
            "Generate stub file of a module `{name}` at {dest}",
            dest = dest.display()
        );
    }
    Ok(())
}
```

**Key Simplifications:**

1. **Path normalization** happens first (dashes → underscores, dots → slashes)
2. **Layout-based decision**: No conditional logic based on submodules or directory existence
3. **Pure Rust**: Always generates `{package_name}.pyi` (uses first segment of module name)
4. **Mixed**: Always generates `{path}/__init__.pyi` (directory-based)
5. **Directory creation** is automatic - parent directories are created as needed
6. **Module formatting** is handled by the `Display` trait implementation of `Module`

## Related PRs and Changes

### Simplified Stub Generation Strategy

**Motivation**: The original PR #348 proposed adding directory existence checking to allow users to control stub file format by pre-creating directories. However, this approach added complexity and didn't fully align with maturin's constraints.

**Evolution to current approach**: Instead of checking directory existence, we simplified the logic to depend **solely on layout type**:

**Old (complex) approach:**
```rust
// Decision based on: submodules + directory existence
let dest = if module.submodules.is_empty() && !self.python_root.join(&path).is_dir() {
    self.python_root.join(format!("{path}.pyi"))
} else {
    self.python_root.join(path).join("__init__.pyi")
};
```

**New (simplified) approach:**
```rust
// Decision based on: layout type only
let dest = if self.is_mixed_layout {
    // Mixed: Always directory-based
    self.python_root.join(&path).join("__init__.pyi")
} else {
    // Pure: Always single file
    let package_name = normalized_name.split('.').next().unwrap();
    self.python_root.join(format!("{package_name}.pyi"))
};
```

**Benefits:**

1. **Aligns with maturin constraints**: Pure Rust layout always generates single file (what maturin packages), mixed layout always generates directory structure (what maturin supports for submodules)
2. **Eliminates ambiguity**: No conditional logic based on runtime state (directory existence, submodule count)
3. **Predictable behavior**: Same configuration always produces same file structure
4. **Enforces best practices**: Projects with submodules must declare mixed layout, ensuring proper maturin packaging

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
