# pyo3-stub-gen 

[![DeepWiki](https://img.shields.io/badge/DeepWiki-Jij--Inc%2Fpyo3--stub--gen-blue.svg?logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACwAAAAyCAYAAAAnWDnqAAAAAXNSR0IArs4c6QAAA05JREFUaEPtmUtyEzEQhtWTQyQLHNak2AB7ZnyXZMEjXMGeK/AIi+QuHrMnbChYY7MIh8g01fJoopFb0uhhEqqcbWTp06/uv1saEDv4O3n3dV60RfP947Mm9/SQc0ICFQgzfc4CYZoTPAswgSJCCUJUnAAoRHOAUOcATwbmVLWdGoH//PB8mnKqScAhsD0kYP3j/Yt5LPQe2KvcXmGvRHcDnpxfL2zOYJ1mFwrryWTz0advv1Ut4CJgf5uhDuDj5eUcAUoahrdY/56ebRWeraTjMt/00Sh3UDtjgHtQNHwcRGOC98BJEAEymycmYcWwOprTgcB6VZ5JK5TAJ+fXGLBm3FDAmn6oPPjR4rKCAoJCal2eAiQp2x0vxTPB3ALO2CRkwmDy5WohzBDwSEFKRwPbknEggCPB/imwrycgxX2NzoMCHhPkDwqYMr9tRcP5qNrMZHkVnOjRMWwLCcr8ohBVb1OMjxLwGCvjTikrsBOiA6fNyCrm8V1rP93iVPpwaE+gO0SsWmPiXB+jikdf6SizrT5qKasx5j8ABbHpFTx+vFXp9EnYQmLx02h1QTTrl6eDqxLnGjporxl3NL3agEvXdT0WmEost648sQOYAeJS9Q7bfUVoMGnjo4AZdUMQku50McDcMWcBPvr0SzbTAFDfvJqwLzgxwATnCgnp4wDl6Aa+Ax283gghmj+vj7feE2KBBRMW3FzOpLOADl0Isb5587h/U4gGvkt5v60Z1VLG8BhYjbzRwyQZemwAd6cCR5/XFWLYZRIMpX39AR0tjaGGiGzLVyhse5C9RKC6ai42ppWPKiBagOvaYk8lO7DajerabOZP46Lby5wKjw1HCRx7p9sVMOWGzb/vA1hwiWc6jm3MvQDTogQkiqIhJV0nBQBTU+3okKCFDy9WwferkHjtxib7t3xIUQtHxnIwtx4mpg26/HfwVNVDb4oI9RHmx5WGelRVlrtiw43zboCLaxv46AZeB3IlTkwouebTr1y2NjSpHz68WNFjHvupy3q8TFn3Hos2IAk4Ju5dCo8B3wP7VPr/FGaKiG+T+v+TQqIrOqMTL1VdWV1DdmcbO8KXBz6esmYWYKPwDL5b5FA1a0hwapHiom0r/cKaoqr+27/XcrS5UwSMbQAAAABJRU5ErkJggg==)](https://deepwiki.com/Jij-Inc/pyo3-stub-gen)

Python stub file (`*.pyi`) generator for [PyO3] with [maturin] projects.

[PyO3]: https://github.com/PyO3/pyo3
[maturin]: https://github.com/PyO3/maturin

| crate name | crates.io | docs.rs | doc (main) |
| --- | --- | --- | --- |
| [pyo3-stub-gen] | [![crate](https://img.shields.io/crates/v/pyo3-stub-gen.svg)](https://crates.io/crates/pyo3-stub-gen)  | [![docs.rs](https://docs.rs/pyo3-stub-gen/badge.svg)](https://docs.rs/pyo3-stub-gen) | [![doc (main)](https://img.shields.io/badge/doc-main-blue?logo=github)](https://jij-inc.github.io/pyo3-stub-gen/pyo3_stub_gen/index.html) |
| [pyo3-stub-gen-derive] | [![crate](https://img.shields.io/crates/v/pyo3-stub-gen-derive.svg)](https://crates.io/crates/pyo3-stub-gen-derive)  | [![docs.rs](https://docs.rs/pyo3-stub-gen-derive/badge.svg)](https://docs.rs/pyo3-stub-gen-derive) | [![doc (main)](https://img.shields.io/badge/doc-main-blue?logo=github)](https://jij-inc.github.io/pyo3-stub-gen/pyo3_stub_gen_derive/index.html) |

[pyo3-stub-gen]: ./pyo3-stub-gen/
[pyo3-stub-gen-derive]: ./pyo3-stub-gen-derive/

> [!NOTE]
> Minimum supported Python version is 3.10. Do not enable 3.9 or older in PyO3 setting.

# Design
Our goal is to create a stub file `*.pyi` from Rust code, however,
automated complete translation is impossible due to the difference between Rust and Python type systems and the limitation of proc-macro.
We take semi-automated approach:

- Provide a default translator which will work **most** cases, not **all** cases
- Also provide a manual way to specify the translation.

If the default translator does not work, users can specify the translation manually,
and these manual translations can be integrated with what the default translator generates.
So the users can use the default translator as much as possible and only specify the translation for the edge cases.

[pyo3-stub-gen] crate provides the manual way to specify the translation,
and [pyo3-stub-gen-derive] crate provides the default translator as proc-macro based on the mechanism of [pyo3-stub-gen].

# Usage

If you are looking for a working example, please see the [examples](./examples/) directory.

| Example          | Description |
|:-----------------|:------------|
| [examples/pure]  | Example for [Pure Rust maturin project](https://www.maturin.rs/project_layout#pure-rust-project) |
| [examples/mixed] | Example for [Mixed Rust/Python maturin project](https://www.maturin.rs/project_layout#mixed-rustpython-project) |
| [examples/mixed_sub] | Example for [Mixed Rust/Python maturin project](https://www.maturin.rs/project_layout#mixed-rustpython-project) with submodule |

[examples/pure]: ./examples/pure/
[examples/mixed]: ./examples/mixed/
[examples/mixed_sub]: ./examples/mixed_sub/

Here we describe basic usage of [pyo3-stub-gen] crate based on [examples/pure] example.

## Annotate Rust code with proc-macro

This crate provides a procedural macro `#[gen_stub_pyfunction]` and others to generate a Python stub file.
It is used with PyO3's `#[pyfunction]` macro. Let's consider a simple example PyO3 project:

```rust
use pyo3::prelude::*;

#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pymodule]
fn your_module_name(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    Ok(())
}
```

To generate a stub file for this project, please modify it as follows:

```rust
use pyo3::prelude::*;
use pyo3_stub_gen::{derive::gen_stub_pyfunction, define_stub_info_gatherer};

#[gen_stub_pyfunction]  // Proc-macro attribute to register a function to stub file generator.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pymodule]
fn your_module_name(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    Ok(())
}

// Define a function to gather stub information.
define_stub_info_gatherer!(stub_info);
```

> [!NOTE]
> The `#[gen_stub_pyfunction]` macro must be placed before `#[pyfunction]` macro.

### `#[gen_stub(skip)]`

For functions or methods that you want to exclude from the generated stub file, use the `#[gen_stub(skip)]` attribute:

```rust
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass]
#[pyclass]
struct MyClass;

#[gen_stub_pymethods]
#[pymethods]
impl MyClass {
    #[gen_stub(skip)]
    fn internal_method(&self) {
        // This method will not appear in the .pyi file
    }
}
```

### `#[gen_stub(default=xx)]`

For getters, setters, and class attributes, you can specify default values that will appear in the stub file:

```rust
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass]
#[pyclass]
struct Config {
    #[pyo3(get, set)]
    #[gen_stub(default = Config::default().timeout)]
    timeout: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config { timeout: 30 }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl Config {
    #[getter]
    #[gen_stub(default = Config::default().timeout)]
    fn get_timeout(&self) -> usize {
        self.timeout
    }
}
```

## Generate a stub file

And then, create an executable target in [`src/bin/stub_gen.rs`](./examples/pure/src/bin/stub_gen.rs) to generate a stub file:

```rust:ignore
use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    // `stub_info` is a function defined by `define_stub_info_gatherer!` macro.
    let stub = pure::stub_info()?;
    stub.generate()?;
    Ok(())
}
```

and add `rlib` in addition to `cdylib` in `[lib]` section of `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib", "rlib"]
```

This target generates a stub file [`pure.pyi`](./examples/pure/pure.pyi) when executed.

```shell
cargo run --bin stub_gen
```

The stub file is automatically found by `maturin`, and it is included in the wheel package. See also the [maturin document](https://www.maturin.rs/project_layout#adding-python-type-information) for more details.

## Manual Overriding

When the automatic Rust-to-Python type translation doesn't produce the desired result, you can manually specify type information using Python stub syntax. There are two main approaches:

1. **Complete override** - Replace entire function signature with `#[gen_stub_pyfunction(python = "...")]`
2. **Partial override** - Override specific arguments or return types with `#[gen_stub(override_type(...))]`

### Method 1: Complete Override Using `python` Parameter

Use the `python` parameter to specify the complete function signature in Python stub syntax. This is ideal when you need to define complex types or when the entire signature needs custom definition.

```rust
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyfunction(python = r#"
    import collections.abc
    import typing

    def fn_with_callback(callback: collections.abc.Callable[[str], typing.Any]) -> collections.abc.Callable[[str], typing.Any]:
        """Example using python parameter for complete override."""
"#)]
#[pyfunction]
pub fn fn_with_callback<'a>(callback: Bound<'a, PyAny>) -> PyResult<Bound<'a, PyAny>> {
    callback.call1(("Hello!",))?;
    Ok(callback)
}
```

This approach:
- ✅ Provides complete control over the generated stub
- ✅ Supports complex types like `collections.abc.Callable`
- ✅ Allows adding custom docstrings
- ✅ Import statements are automatically extracted

### Method 2: Partial Override Using Attributes

For selective overrides, use `#[gen_stub(override_type(...))]` on specific arguments or `#[gen_stub(override_return_type(...))]` on the function. This is useful when most types translate correctly but a few need adjustment.

```rust
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyfunction]
#[pyfunction]
#[gen_stub(override_return_type(type_repr="collections.abc.Callable[[str], typing.Any]", imports=("collections.abc", "typing")))]
pub fn get_callback<'a>(
    #[gen_stub(override_type(type_repr="collections.abc.Callable[[str], typing.Any]", imports=("collections.abc", "typing")))]
    cb: Bound<'a, PyAny>,
) -> PyResult<Bound<'a, PyAny>> {
    Ok(cb)
}
```

This approach:
- ✅ Fine-grained control over individual types
- ✅ Preserves automatic generation for other parameters
- ✅ Explicit about which types need manual specification

### Method 3: Separate Definitions Using Macros

**How `submit!` works:**

The `#[gen_stub_pyfunction]` and `#[gen_stub_pyclass]` macros automatically generate `submit!` blocks internally to register type information. You can also manually add `submit!` blocks to supplement or override this automatic registration.

When multiple `submit!` blocks exist for the same function or method, the stub generator interprets them as overloads and generates `@overload` decorators in the `.pyi` file. This enables proper type checking for functions that accept multiple type signatures.

**Use cases:**

For function overloads or when you want to keep the Python stub definition separate from the Rust implementation, use `gen_function_from_python!` or `gen_methods_from_python!` macros with `submit!` blocks.

**Function overloads:**

```rust
use pyo3::prelude::*;
use pyo3_stub_gen::{derive::*, inventory::submit};

// #[gen_stub_pyfunction] automatically generates one submit! for float signature
#[gen_stub_pyfunction]
#[pyfunction]
pub fn process(x: f64) -> f64 {
    x + 1.0
}

// Manual submit! for integer overload
// Now we have 2 submit! blocks for "process" → generates @overload decorators
submit! {
    gen_function_from_python! {
        r#"
        def process(x: int) -> int:
            """Process integer input"""
        "#
    }
}
```

**Class method overloads:**

```rust
use pyo3::prelude::*;
use pyo3_stub_gen::{derive::*, inventory::submit};

#[gen_stub_pyclass]
#[pyclass]
pub struct Calculator {}

// #[gen_stub_pymethods] automatically generates submit! for float signature
#[gen_stub_pymethods]
#[pymethods]
impl Calculator {
    fn add(&self, x: f64) -> f64 {
        x + 1.0
    }
}

// Manual submit! for integer overload
// Now Calculator.add has 2 submit! blocks → generates @overload decorators
submit! {
    gen_methods_from_python! {
        r#"
        class Calculator:
            def add(self, x: int) -> int:
                """Add integer (overload)"""
        "#
    }
}
```

This approach:
- ✅ Ideal for `@overload` decorator support
- ✅ Keeps type definitions organized separately
- ✅ Allows multiple signatures for the same function

### Advanced: Using `RustType` Marker

Within Python stub syntax, you can reference Rust types directly using the `pyo3_stub_gen.RustType["TypeName"]` marker. This leverages the `PyStubType` trait implementation of the Rust type.

```rust
use pyo3::prelude::*;
use pyo3_stub_gen::{derive::*, inventory::submit};

#[pyfunction]
pub fn sum_list(values: Vec<i32>) -> i32 {
    values.iter().sum()
}

submit! {
    gen_function_from_python! {
        r#"
        def sum_list(values: pyo3_stub_gen.RustType["Vec<i32>"]) -> pyo3_stub_gen.RustType["i32"]:
            """Sum a list of integers"""
        "#
    }
}
```

The `RustType` marker automatically expands to the appropriate Python type:
- `RustType["Vec<i32>"]` → `typing.Sequence[int]` (for arguments)
- `RustType["i32"]` → `int` (for return values)

This is particularly useful for:
- Generic types like `Vec<T>`, `HashMap<K, V>`
- Custom types that implement `PyStubType`
- Ensuring consistency between Rust and Python type mappings

### When to Use Which Method

| Scenario | Recommended Method |
|----------|-------------------|
| Complex types (e.g., `Callable`, `Protocol`) | Method 1: `python = "..."` parameter |
| Override one or two arguments | Method 2: `#[gen_stub(override_type(...))]` |
| Function overloads (`@overload`) | Method 3: `gen_function_from_python!` |
| Reference Rust types in Python syntax | Use `RustType["..."]` marker |
| Complete function signature replacement | Method 1: `python = "..."` parameter |

For complete examples, see the [examples/pure](./examples/pure/) directory, particularly:
- `overriding.rs` - Type override examples
- `overloading.rs` - Function overload examples
- `rust_type_marker.rs` - RustType marker examples

## Advanced: mypy.stubtest integration

[mypy stubtest](https://mypy.readthedocs.io/en/stable/stubtest.html) validates that stub files match runtime behavior. You can add it to your test suite:

```bash
uv run stubtest your_module_name --ignore-missing-stub --ignore-disjoint-bases
```

### Required flags for PyO3/maturin projects

- `--ignore-missing-stub` - Maturin creates internal native modules (`.so` files) that re-export to `__init__.py`. Stubtest looks for stubs for these internal modules, which don't exist (all types are in `__init__.pyi`). This flag prevents false positives.
- `--ignore-disjoint-bases` - PyO3 classes are disjoint bases at runtime, but pyo3-stub-gen does not generate `@typing.disjoint_base` decorators.

### Known limitation: nested submodules

**Stubtest does not work with PyO3 nested submodules.** Nested `#[pymodule]` creates runtime attributes (not importable modules), but stub files use directory structure. For projects with nested submodules, disable stubtest for those packages. See `examples/mixed_sub/Taskfile.yml` for an example.

# Contribution
To be written.

# License

© 2024 Jij Inc.

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

# Links

- [MusicalNinjas/pyo3-stubgen](https://github.com/MusicalNinjas/pyo3-stubgen)
  - Same motivation, but different approach.
  - This project creates a stub file by loading the compiled library and inspecting the `__text_signature__` attribute generated by PyO3 in Python side.
- [pybind11-stubgen](https://github.com/sizmailov/pybind11-stubgen)
  - Stub file generator for [pybind11](https://github.com/pybind/pybind11) based C++ projects.
