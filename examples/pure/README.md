# Pure Rust example

This example exposes a Python module named `pure` and commits its generated
[`pure.pyi`](pure.pyi) and [API reference](docs/api/api_reference.json).

[`src/doc_attributes.rs`](src/doc_attributes.rs) demonstrates sharing
[`src/docs/external.md`](src/docs/external.md) between runtime docstrings,
stubs, and API documentation using `#[doc = include_str!(...)]`. It also mixes
doc comments, literal attributes, and `concat!` in declaration order, including
documentation on fields, properties, class attributes, and enum variants.
The existing `NumberRenameAll` in [`src/lib.rs`](src/lib.rs) covers simple enums.
The Markdown contains a table and a Python doctest that exercises the module.

From the repository root, run `task pure:stub-gen` to regenerate the outputs,
`task pure:test` to run the Python tests and type checkers, and
`task pure:doc-test` to build the Sphinx documentation.
