# pyo3-stub-gen

Python stub file (`*.pyi`) generator for [PyO3] with [maturin] projects.

[PyO3]: https://github.com/PyO3/pyo3
[maturin]: https://github.com/PyO3/maturin

| crate name | crates.io | docs.rs | doc (main) |
| --- | --- | --- | --- |
| [pyo3-stub-gen] | [![crate](https://img.shields.io/crates/v/pyo3-stub-gen.svg)](https://crates.io/crates/pyo3-stub-gen)  | [![docs.rs](https://docs.rs/pyo3-stub-gen/badge.svg)](https://docs.rs/pyo3-stub-gen) | [![doc (main)](https://img.shields.io/badge/doc-main-blue?logo=github)](https://jij-inc.github.io/pyo3-stub-gen/pyo3_stub_gen/index.html) |
| [pyo3-stub-gen-derive] | [![crate](https://img.shields.io/crates/v/pyo3-stub-gen-derive.svg)](https://crates.io/crates/pyo3-stub-gen-derive)  | [![docs.rs](https://docs.rs/pyo3-stub-gen-derive/badge.svg)](https://docs.rs/pyo3-stub-gen-derive) | [![doc (main)](https://img.shields.io/badge/doc-main-blue?logo=github)](https://jij-inc.github.io/pyo3-stub-gen/pyo3_stub_gen_derive/index.html) |

[pyo3-stub-gen]: ./pyo3-stub-gen/
[pyo3-stub-gen-derive]: ./pyo3-stub-gen-derive/

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

## Generate a stub file

And then, create an executable target in [`src/bin/stub_gen.rs`](./examples/pure/src/bin/stub_gen.rs) to generate a stub file:

```rust
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
