# Type Statement Alias Example

This example demonstrates the Python 3.12+ `type` statement syntax for type aliases.

## Configuration

The `pyproject.toml` file includes:

```toml
[tool.pyo3-stub-gen]
use-type-statement = true
```

This generates type aliases using the `type` statement instead of `TypeAlias`:

```python
# Generated with use-type-statement = true
type SimpleAlias = int | None
type StrIntMap = dict[str, int]
```

Compare this to the default syntax (see `examples/pure`):

```python
# Generated with default settings
from typing import TypeAlias
SimpleAlias: TypeAlias = int | None
StrIntMap: TypeAlias = dict[str, int]
```

## Parser Support

The parser accepts **both** syntaxes in Python-style submissions:

```rust
// Pre-3.12 syntax (still accepted)
gen_type_alias_from_python!(
    "module",
    r#"
    from typing import TypeAlias
    CallbackType: TypeAlias = collections.abc.Callable[[str], None]
    "#
);

// Python 3.12+ syntax (also accepted)
gen_type_alias_from_python!(
    "module",
    r#"
    type OptionalCallback = collections.abc.Callable[[str], None] | None
    "#
);
```

Both inputs will be output according to the `use-type-statement` configuration.

## Running

Generate stub files:

```bash
cargo run --bin stub_gen
```

Run tests:

```bash
maturin develop
pytest
```
