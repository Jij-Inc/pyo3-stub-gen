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

Use uv 0.12.21 or later and Task from the repository root:

```bash
UV_PYTHON=3.12 task type-statement-alias:stub-gen type-statement-alias:test
```

This example is a conditional editable path dependency in the root `tests`
dependency group, included by `dev`. It shares the root `.venv` and `uv.lock`,
but is excluded from the uv workspace so that its `requires-python = ">=3.12"`
does not prevent other examples from running on Python 3.10 and 3.11.

The dependency marker enables this package only on Python 3.12 and later.
The test task skips this example on older interpreters, including when called
through `task test`. Run uv commands for this example with `--project ../..`
from this directory to use the shared environment instead of creating an
independent project environment.
