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
