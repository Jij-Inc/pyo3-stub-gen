# pyo3-stub-gen

[![Document](https://img.shields.io/badge/document-main-blue?logo=github)](https://jij-inc.github.io/pyo3-stub-gen/pyo3_stub_gen/index.html)

Python stub file (`*.pyi`) generator for PyO3 based projects.

# Usage

This crate provides a procedural macro `#[gen_stub_pyfunction]` and others to generate a Python stub file.
It is used with PyO3's `#[pyfunction]` macro. Let's consider a simple example PyO3 project:

```rust
use pyo3::prelude::*;

#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pymodule]
fn pyo3_stub_gen_testing(_py: Python, m: &PyModule) -> PyResult<()> {
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
fn pyo3_stub_gen_testing(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    Ok(())
}

// Define a function to gather stub information.
define_stub_info_gatherer!(stub_info);
```

And then, create an executable target in `src/bin/stub_gen.rs`:

```rust
use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    // `stub_info` is a function defined by `define_stub_info_gatherer!` macro.
    let stub = pyo3_stub_gen_testing_pure::stub_info()?;
    stub.generate()?;
    Ok(())
}
```

and add `rlib` in addition to `cdylib` in `[lib]` section of `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib", "rlib"]
```

This target generates a stub file `${CARGO_MANIFEST_DIR}/pyo3_stub_gen_testing.pyi` when executed.

```shell
cargo run --bin stub_gen
```

The stub file is automatically found by `maturin`, and it is included in the wheel package. See also the [maturin document](https://www.maturin.rs/project_layout#adding-python-type-information) for more details.

There is a working example at [pyo3-stub-gen-testing-pure](./pyo3-stub-gen-testing-pure/) directory with [generated stub file](./pyo3-stub-gen-testing-pure/pyo3_stub_gen_testing_pure.pyi).

# License

© 2024 Jij Inc.

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.
