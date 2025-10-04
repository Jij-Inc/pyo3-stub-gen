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

## Advanced: `#[gen_stub(xxx)]` Attributes
### `#[gen_stub(default=xx)]`

For getters, setters, and classattr functions, you can specify the default value of it. e.g.
```rust
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass]
#[pyclass]
struct A {
    #[pyo3(get,set)]
    #[gen_stub(default = A::default().x)]
    x: usize,
    y: usize,
}

impl Default for A {
    fn default() -> Self {
        A { x: 0, y: 0 }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl A {
    #[gen_stub(default = A::default().y)]
    fn get_y(&self) -> usize {
        self.y
    }
}
```

### `#[gen_stub(skip)]`
For classattrs or functions in pymethods, ignore it in .pyi file. e.g.
```rust
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass]
#[pyclass]
struct A;

#[gen_stub_pymethods]
#[pymethods]
impl A {
    #[gen_stub(skip)]
    fn need_skip(&self) {}
}
```

### `#[gen_stub(override_type(type_repr=xx, imports=(xx)))]` and `#[gen_stub(override_return_type(type_repr=xx, imports=(xx)))]`
Override the type for function arguments or return type in .pyi file. e.g.
```rust
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyfunction]
#[pyfunction]
#[gen_stub(override_return_type(type_repr="typing.Never", imports=("typing")))]
fn say_hello_forever<'a>(
    #[gen_stub(override_type(type_repr="collections.abc.Callable[[str]]", imports=("collections.abc")))]
    cb: Bound<'a, PyAny>,
) -> PyResult<()> {
    loop {
        cb.call1(("Hello!",))?;
    }
}
```

## Advanced: mypy.stubtest integration

### What is stubtest?

[mypy stubtest](https://mypy.readthedocs.io/en/stable/stubtest.html) is a tool that validates Python stub files (`.pyi`) against the actual runtime behavior of your module. It ensures that the type annotations in your stubs accurately reflect what's actually available at runtime.

### Using stubtest with pyo3-stub-gen

You can add stubtest to your test suite to validate the generated stub files:

```bash
uv run stubtest your_module_name --ignore-missing-stub --ignore-disjoint-bases
```

The following flags are **recommended** for PyO3/maturin projects:
- `--ignore-missing-stub` - Ignore missing stubs for internal native modules (see details below)
- `--ignore-disjoint-bases` - Ignore `@typing.disjoint_base` decorator requirements (currently not supported by pyo3-stub-gen)

Here's why these flags are necessary:

### Why `--ignore-missing-stub` is necessary

#### The maturin module structure

When maturin builds a pure Rust project, it creates the following structure:

```text
your_package/
├── __init__.py              # Re-exports from the native module
├── __init__.pyi             # Generated stub file (your_package.pyi → __init__.pyi)
└── your_package.cpython-*.so  # Native extension module
```

The `__init__.py` file typically contains:

```python
from .your_package import *
```

This re-exports everything from the native `.so` module. However, stubtest will try to find a stub file named `your_package.pyi` for the native module `.your_package`, which doesn't exist (and shouldn't exist, since all type information is already in `__init__.pyi`).

#### The error without `--ignore-missing-stub`

Without the flag, stubtest reports errors like:

```text
error: your_package.your_package failed to find stubs
```

This is a false positive - the stub file exists as `__init__.pyi`, but stubtest is looking for a stub for the internal native module created by maturin.

#### Solution

Use `--ignore-missing-stub` to ignore missing stubs for internal native modules:

```yaml
# Taskfile.yml example
test:
  cmds:
    - uv run pytest
    - uv run pyright
    - uv run mypy --show-error-codes -p your_package
    - uv run stubtest your_package --ignore-missing-stub --ignore-disjoint-bases
```

This flag tells stubtest to ignore cases where a runtime module doesn't have a corresponding stub file, which is expected for the internal native modules that maturin creates as an implementation detail.

### Why `--ignore-disjoint-bases` is necessary

#### What is `@typing.disjoint_base`?

The `@typing.disjoint_base` decorator (introduced in [PEP 800](https://peps.python.org/pep-0800/)) marks classes that cannot be used together in multiple inheritance. For example, if class `A` and class `B` are both marked as `@disjoint_base`, then a class cannot inherit from both `A` and `B` simultaneously.

PyO3 classes are typically disjoint bases because they are implemented in Rust and have incompatible internal layouts. However, pyo3-stub-gen currently does not generate the `@typing.disjoint_base` decorator in stub files.

#### Current status and future plans

**Current workaround**: Use `--ignore-disjoint-bases` to suppress these errors in stubtest.

**Future plan**: We plan to add support for generating `@typing.disjoint_base` decorators in a future version of pyo3-stub-gen. Until then, please use the `--ignore-disjoint-bases` flag to avoid false positives in stubtest.

### Known stubtest limitations

#### PyO3 nested submodules

**Stubtest does not work properly with PyO3 nested submodules** (created with nested `#[pymodule]` attributes).

When you create nested modules in PyO3:

```rust
#[pymodule]
fn main_mod(m: &Bound<'_, PyModule>) -> PyResult<()> {
    #[pymodule]
    fn submod(m: &Bound<'_, PyModule>) -> PyResult<()> {
        // ...
    }
    m.add_wrapped(wrap_pymodule!(submod))?;
    Ok(())
}
```

At runtime, `submod` exists as an **attribute** of `main_mod` (accessible via `main_mod.submod`), but it is **not a separate importable module** (you cannot `import package.main_mod.submod`).

However, pyo3-stub-gen generates stub files in a directory structure:
```text
package/
  main_mod/
    __init__.pyi
    submod.pyi        # Stubtest tries to import this as a module
```

Stubtest sees this file structure and attempts to `import package.main_mod.submod`, which fails because it's not a real submodule. This causes stubtest to skip validation of the entire submodule's contents.

**Workaround**: For projects with nested submodules, **disable stubtest** for that package. Type checking with mypy/pyright still works correctly. See `examples/mixed_sub/Taskfile.yml` for an example.

#### Missing decorators

Currently, pyo3-stub-gen does not generate the following decorators that stubtest expects:

- ✅ `@final` - **Now supported** (as of v0.14.2). PyO3 classes and enums are marked with `@final` since they cannot be subclassed at runtime.
- `@typing.disjoint_base` - PyO3 classes are disjoint bases at runtime, but stubs don't mark them. Use `--ignore-disjoint-bases` to suppress these errors.

These are cosmetic issues that don't affect the practical usability of the generated stubs for type checking with mypy or pyright.

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
