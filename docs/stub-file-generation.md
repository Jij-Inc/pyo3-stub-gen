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

**Usage**: `pyo3-stub-gen/src/generate/stub_info.rs:65-72`

```rust
fn from_pyproject_toml(pyproject: PyProject) -> Self {
    StubInfoBuilder::from_project_root(
        pyproject.module_name().to_string(),
        pyproject
            .python_source()
            .unwrap_or(PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())),
    )
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

The stub file generation logic is implemented in `pyo3-stub-gen/src/generate/stub_info.rs`. For each module, the generator determines the output path based on two factors:

1. **Submodule existence**: Whether the module has submodules defined in Rust code
2. **Directory existence**: Whether a directory already exists at the target path

### Decision Tree

```rust
let dest = if module.submodules.is_empty() && !self.python_root.join(&path).is_dir() {
    self.python_root.join(format!("{path}.pyi"))
} else {
    self.python_root.join(path).join("__init__.pyi")
};
```

The generator creates:

- **Single-file stub** (`module.pyi`): When **both** conditions are true:
  - The module has no submodules (`module.submodules.is_empty()`)
  - No directory exists at the module path (`!self.python_root.join(&path).is_dir()`)

- **Directory-based stub** (`module/__init__.pyi`): When **either** condition is true:
  - The module has submodules, **OR**
  - A directory already exists at the module path

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

## Submodule Detection

Submodules are automatically detected during the build phase in the `register_submodules` method:

```rust
fn register_submodules(&mut self) {
    let mut map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for module in self.modules.keys() {
        let path = module.split('.').collect::<Vec<_>>();
        let n = path.len();
        if n <= 1 {
            continue;
        }
        map.entry(path[..n - 1].join("."))
            .or_default()
            .insert(path[n - 1].to_string());
    }
    for (parent, children) in map {
        if let Some(module) = self.modules.get_mut(&parent) {
            module.submodules.extend(children);
        }
    }
}
```

**Detection Logic:**

1. Parse all registered module names by splitting on `.`
2. For modules with 2+ segments (e.g., `package.main_mod.sub_a`):
   - Parent: All segments except the last (`package.main_mod`)
   - Child: Last segment (`sub_a`)
3. Register each child in its parent's `submodules` set

**Example:**

Given these modules registered via `#[gen_stub_pyfunction(module = "...")]`:
- `mixed_sub.main_mod`
- `mixed_sub.main_mod.mod_a`
- `mixed_sub.main_mod.mod_b`
- `mixed_sub.main_mod.int`

The detector creates:
- `mixed_sub.main_mod.submodules = {"mod_a", "mod_b", "int"}`

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
│       └── main_mod.pyi              # Generated stub
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
- No pre-existing `python/mixed/main_mod/` directory
- **Output**: `python/mixed/main_mod.pyi` (single file)

**Coexistence:**
- `main_mod.cpython-313-darwin.so` (native module file)
- `main_mod.pyi` (stub file)
- Both exist as files in the same directory

### Mixed Sub Layout

**Project Structure:**
```
mixed_sub/
├── python/
│   └── mixed_sub/
│       ├── main_mod.cpython-313-darwin.so
│       └── main_mod/                  # Directory for stubs
│           ├── __init__.pyi           # Generated stub
│           ├── mod_a.pyi              # Generated stub
│           ├── mod_b.pyi              # Generated stub
│           └── int.pyi                # Generated stub
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
  - `python/mixed_sub/main_mod/mod_a.pyi` (submodule)
  - `python/mixed_sub/main_mod/mod_b.pyi` (submodule)
  - `python/mixed_sub/main_mod/int.pyi` (submodule)

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

## Directory Pre-creation Pattern

The directory existence check (`!self.python_root.join(&path).is_dir()`) enables a powerful pattern: **users can control stub generation format by pre-creating directories**.

### Use Case

Even without Rust-defined submodules, you can force `__init__.pyi` generation for better IDE support:

**Before:**
```
python/
└── my_package/
    └── my_module.pyi      # Single file stub
```

**After creating directory:**
```bash
mkdir -p python/my_package/my_module
```

**Result:**
```
python/
└── my_package/
    ├── my_module.cpython-313-darwin.so
    └── my_module/
        └── __init__.pyi   # Directory-based stub
```

**Benefits:**
- Better organization for future submodules
- Improved static analyzer understanding (Pylance, Pyright)
- Consistent structure across mixed Python/Rust projects

### Why This Matters

Some static analysis tools (like Pylance) better understand module hierarchies when they follow directory structures, even for single-file modules. Pre-creating directories reduces "source cannot be found" warnings.

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

3. **Determine `python_root`** (`stub_info.rs:65-72`):
   ```rust
   fn from_pyproject_toml(pyproject: PyProject) -> Self {
       StubInfoBuilder::from_project_root(
           pyproject.module_name().to_string(),
           pyproject
               .python_source()  // Returns Some(path) for mixed, None for pure
               .unwrap_or(PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())),
       )
   }
   ```

**The flow:**
- Mixed layout: `python_source()` → `Some("python")` → `python_root = <project>/python`
- Pure layout: `python_source()` → `None` → `python_root = CARGO_MANIFEST_DIR`

### File Creation Process

**Location**: `pyo3-stub-gen/src/generate/stub_info.rs:31-55`

```rust
pub fn generate(&self) -> Result<()> {
    for (name, module) in self.modules.iter() {
        // 1. Normalize module name
        let normalized_name = name.replace("-", "_");
        let path = normalized_name.replace(".", "/");

        // 2. Determine destination path
        let dest = if module.submodules.is_empty() && !self.python_root.join(&path).is_dir() {
            self.python_root.join(format!("{path}.pyi"))
        } else {
            self.python_root.join(path).join("__init__.pyi")
        };

        // 3. Create parent directories if needed
        let dir = dest.parent().context("Cannot get parent directory")?;
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }

        // 4. Write stub file
        let mut f = fs::File::create(&dest)?;
        write!(f, "{module}")?;
        log::info!("Generate stub file of a module `{name}` at {dest}", dest = dest.display());
    }
    Ok(())
}
```

**Key Points:**

1. **Path normalization** happens first (dashes → underscores, dots → slashes)
2. **Decision logic** considers both submodules and directory existence
3. **Directory creation** is automatic - parent directories are created as needed
4. **Module formatting** is handled by the `Display` trait implementation of `Module`

## Related PRs and Changes

### PR #348: Directory-aware stub generation

**Change:** Added directory existence check to generation logic

**Before:**
```rust
let dest = if module.submodules.is_empty() {
    self.python_root.join(format!("{path}.pyi"))
} else {
    self.python_root.join(path).join("__init__.pyi")
};
```

**After:**
```rust
let dest = if module.submodules.is_empty() && !self.python_root.join(&path).is_dir() {
    self.python_root.join(format!("{path}.pyi"))
} else {
    self.python_root.join(path).join("__init__.pyi")
};
```

**Impact:**

- **Pure layout**: No change (no directories exist)
- **Mixed layout without submodules**: Can be controlled by pre-creating directories
- **Mixed layout with submodules**: No change (already generates `__init__.pyi`)

**Motivation:** Allows users to pre-create project hierarchies that static analysis tools understand better, reducing warnings about missing sources.

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

7. **Pre-create directories for IDE support**: If your IDE shows warnings, create module directories before stub generation

### Layout Detection Verification

8. **Verify your layout is correctly detected**: Check where stub files are generated
   ```bash
   # Pure Rust: stubs at project root
   ls *.pyi

   # Mixed: stubs in python-source directory
   ls python/**/*.pyi
   ```

9. **If stubs are in the wrong location**: Check your `pyproject.toml` configuration
   ```toml
   [tool.maturin]
   python-source = "python"  # Add this for mixed layout
   ```

### Common Pitfalls

10. **❌ Do NOT use pure Rust layout with submodules**
    ```rust
    // This will generate stubs, but maturin will NOT package them!
    #[gen_stub_pyfunction(module = "mymodule.sub")]
    #[pyfunction]
    fn my_function() { ... }
    ```

    **Problem**: pyo3-stub-gen will generate `mymodule/__init__.pyi` and `mymodule/sub.pyi`, but maturin will only package `mymodule.pyi` (if it exists) and ignore the directory.

    **Solution**: Add `python-source = "python"` to your `pyproject.toml` and move your project to mixed layout:
    ```bash
    mkdir -p python/mymodule
    # Re-run stub generation - stubs will now be in python/mymodule/
    cargo run --bin stub_gen
    ```

## See Also

- [Architecture](./architecture.md) - Overall system design
- [Type System](./type-system.md) - Rust to Python type mappings
- [Python Stub Syntax](./python-stub-syntax.md) - Manual stub syntax specification
