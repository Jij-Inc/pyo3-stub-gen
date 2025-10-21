# Type System

## Overview

pyo3-stub-gen provides automatic type translation from Rust types to Python type annotations. This document describes the type mapping system, how it works internally, and how to extend it for custom types.

## Core Trait: `PyStubType`

All type mappings are defined through the `PyStubType` trait:

```rust
pub trait PyStubType {
    /// Python type representation for function arguments
    fn type_input() -> TypeInfo;

    /// Python type representation for function return values
    fn type_output() -> TypeInfo;
}
```

### Input vs Output Types

The distinction between `type_input()` and `type_output()` exists because some types have different representations depending on usage:

**Example: `Vec<T>`**
- **Input (argument)**: `typing.Sequence[T]` (accepts any sequence-like object)
- **Output (return)**: `list[T]` (always returns a list)

This follows Python's typing conventions where:
- Function parameters should accept broader types (Liskov Substitution Principle)
- Return types should be more specific

**Example: `HashMap<K, V>`**
- **Input**: `typing.Mapping[K, V]` (accepts dict-like objects)
- **Output**: `dict[K, V]` (returns a dict)

For most types, input and output are identical:
```rust
impl PyStubType for i32 {
    fn type_input() -> TypeInfo { TypeInfo::builtin("int") }
    fn type_output() -> TypeInfo { TypeInfo::builtin("int") }
}
```

## Type Mapping Categories

### 1. Built-in Types (`stub_type/builtins.rs`)

Maps Rust primitive types to Python built-in types.

| Rust Type | Python Type (Input/Output) | Notes |
|-----------|---------------------------|-------|
| `bool` | `bool` | Boolean type |
| `i8`, `i16`, `i32`, `i64`, `i128` | `int` | Signed integers |
| `u8`, `u16`, `u32`, `u64`, `u128` | `int` | Unsigned integers |
| `isize`, `usize` | `int` | Platform-dependent integers |
| `f32`, `f64` | `float` | Floating-point numbers |
| `char` | `str` | Single character → single-char string |
| `String`, `&str` | `str` | String types |
| `()` | `None` | Unit type → None |

**Special Types:**

| Rust Type | Input | Output | Notes |
|-----------|-------|--------|-------|
| `Option<T>` | `T \| None` | `T \| None` | Optional values (Python 3.10+ syntax) |
| `Result<T, E>` | `T` | `T` | Error handling (E is ignored in stubs) |
| `PathBuf`, `&Path` | `str` | `str` | File system paths as strings |
| `Cow<'_, T>` | Same as `T` | Same as `T` | Copy-on-write → underlying type |

### 2. Collection Types (`stub_type/collections.rs`)

Maps Rust collections to Python collection types.

| Rust Type | Input | Output | Rationale |
|-----------|-------|--------|-----------|
| `Vec<T>` | `typing.Sequence[T]` | `list[T]` | Accept any sequence, return list |
| `&[T]` | `typing.Sequence[T]` | `list[T]` | Slice → sequence |
| `[T; N]` | `typing.Sequence[T]` | `list[T]` | Fixed-size array |
| `HashSet<T>` | `typing.Set[T]` | `set[T]` | Set types |
| `BTreeSet<T>` | `typing.Set[T]` | `set[T]` | Ordered set → set |
| `HashMap<K, V>` | `typing.Mapping[K, V]` | `dict[K, V]` | Accept any mapping, return dict |
| `BTreeMap<K, V>` | `typing.Mapping[K, V]` | `dict[K, V]` | Ordered map → dict |
| `(T1, T2, ...)` | `tuple[T1, T2, ...]` | `tuple[T1, T2, ...]` | Tuples (up to 12 elements) |

**Generic Type Constraints:**

All collection type parameters must themselves implement `PyStubType`:

```rust
impl<T: PyStubType> PyStubType for Vec<T> {
    fn type_input() -> TypeInfo {
        TypeInfo::with_generic("typing.Sequence", vec![T::type_input()])
    }
    fn type_output() -> TypeInfo {
        TypeInfo::with_generic("list", vec![T::type_output()])
    }
}
```

### 3. PyO3-Specific Types (`stub_type/pyo3.rs`)

Maps PyO3's Python object types.

| Rust Type | Python Type | Notes |
|-----------|-------------|-------|
| `Py<T>` | `T` | Python object reference (unwrapped) |
| `PyObject` | `typing.Any` | Untyped Python object |
| `Bound<'py, T>` | `T` | Bound Python object |
| `Borrowed<'a, 'py, T>` | `T` | Borrowed reference |
| `&PyAny` | `typing.Any` | Any Python type |
| `&PyDict` | `dict[typing.Any, typing.Any]` | Python dict |
| `&PyList` | `list[typing.Any]` | Python list |
| `&PyTuple` | `tuple[typing.Any, ...]` | Python tuple |
| `&PySet`, `&PyFrozenSet` | `set[typing.Any]` | Python sets |
| `&PyString` | `str` | Python string |
| `&PyBytes`, `&PyByteArray` | `bytes` | Python bytes |
| `&PyInt` | `int` | Python integer |
| `&PyFloat` | `float` | Python float |
| `&PyBool` | `bool` | Python boolean |

**PyO3 Lifetime Handling:**

PyO3 types with lifetimes (`'py`, `'a`) are stripped in the Python type representation:
```rust
// Rust: Bound<'py, PyDict>
// Python: dict[Any, Any]
```

### 4. NumPy Types (`stub_type/numpy.rs`)

Requires `numpy` feature flag.

| Rust Type | Python Type | Notes |
|-----------|-------------|-------|
| `PyArray1<T>` | `numpy.ndarray[typing.Any, numpy.dtype[T]]` | 1D array |
| `PyArray2<T>` | `numpy.ndarray[typing.Any, numpy.dtype[T]]` | 2D array |
| `PyArrayDyn<T>` | `numpy.ndarray[typing.Any, numpy.dtype[T]]` | Dynamic dimension array |

**NumPy dtype mappings:**

| Rust Type | NumPy dtype |
|-----------|-------------|
| `f32` | `numpy.float32` |
| `f64` | `numpy.float64` |
| `i8` | `numpy.int8` |
| `i16` | `numpy.int16` |
| `i32` | `numpy.int32` |
| `i64` | `numpy.int64` |
| `u8` | `numpy.uint8` |
| `u16` | `numpy.uint16` |
| `u32` | `numpy.uint32` |
| `u64` | `numpy.uint64` |
| `bool` | `numpy.bool_` |

### 5. Third-Party Crate Support

#### `either` crate (`stub_type/either.rs`)

Requires `either` feature flag.

| Rust Type | Python Type | Notes |
|-----------|-------------|-------|
| `Either<L, R>` | `L \| R` | Sum type → union |

#### `rust_decimal` crate (`stub_type/rust_decimal.rs`)

Requires `rust_decimal` feature flag.

| Rust Type | Python Type | Notes |
|-----------|-------------|-------|
| `Decimal` | `decimal.Decimal` | High-precision decimal |

## Type Representations

### TypeInfo Enum

The `TypeInfo` enum represents all possible type representations:

```rust
pub enum TypeInfo {
    /// Built-in types: int, str, bool, etc.
    BuiltinType(&'static str),

    /// Generic types: list[T], dict[K, V], etc.
    GenericType {
        base: &'static str,
        args: Vec<TypeInfo>,
    },

    /// Union types: T1 | T2 | T3
    UnionType(Vec<TypeInfo>),

    /// Custom class types
    ClassType {
        module: Option<&'static str>,
        name: &'static str,
    },

    /// Manual type override (string representation)
    Override {
        r#type: &'static str,
        imports: &'static [ImportInfo],
    },

    /// Unknown/any type
    Any,
}
```

### Construction Methods

```rust
// Built-in type
TypeInfo::builtin("int")

// Generic type
TypeInfo::with_generic("list", vec![TypeInfo::builtin("int")])

// Union type (T | None)
TypeInfo::union(vec![
    TypeInfo::builtin("int"),
    TypeInfo::none(),
])

// Class type
TypeInfo::class("my_module", "MyClass")

// Any type
TypeInfo::any()
```

### Display Format

`TypeInfo` implements `Display` to generate Python type syntax:

```rust
// int
TypeInfo::builtin("int").to_string() // → "int"

// list[int]
TypeInfo::with_generic("list", vec![TypeInfo::builtin("int")]).to_string()
// → "list[int]"

// int | None
TypeInfo::union(vec![TypeInfo::builtin("int"), TypeInfo::none()]).to_string()
// → "int | None"

// dict[str, typing.Any]
TypeInfo::with_generic("dict", vec![
    TypeInfo::builtin("str"),
    TypeInfo::any(),
]).to_string()
// → "dict[str, typing.Any]"
```

## Implementing PyStubType for Custom Types

### Simple Custom Type

For a custom PyO3 class:

```rust
use pyo3::prelude::*;
use pyo3_stub_gen::{PyStubType, TypeInfo};

#[pyclass]
struct MyClass;

impl PyStubType for MyClass {
    fn type_input() -> TypeInfo {
        TypeInfo::class(Some("my_module"), "MyClass")
    }

    fn type_output() -> TypeInfo {
        TypeInfo::class(Some("my_module"), "MyClass")
    }
}
```

### Generic Custom Type

For a generic wrapper type:

```rust
use pyo3::prelude::*;
use pyo3_stub_gen::{PyStubType, TypeInfo};

#[pyclass]
struct Wrapper<T> {
    value: T,
}

impl<T: PyStubType> PyStubType for Wrapper<T> {
    fn type_input() -> TypeInfo {
        // Accept Wrapper[T] or just T
        TypeInfo::union(vec![
            TypeInfo::with_generic("Wrapper", vec![T::type_input()]),
            T::type_input(),
        ])
    }

    fn type_output() -> TypeInfo {
        // Always return Wrapper[T]
        TypeInfo::with_generic("Wrapper", vec![T::type_output()])
    }
}
```

### Newtype Pattern

For newtypes that should appear as their inner type:

```rust
use pyo3::prelude::*;
use pyo3_stub_gen::{PyStubType, TypeInfo};

struct UserId(i64);

impl PyStubType for UserId {
    fn type_input() -> TypeInfo {
        // Accept int
        TypeInfo::builtin("int")
    }

    fn type_output() -> TypeInfo {
        // Return int
        TypeInfo::builtin("int")
    }
}
```

## Manual Type Overrides

When automatic type translation is insufficient, use manual overrides.

### Attribute-Based Override

```rust
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyfunction]
#[pyfunction]
#[gen_stub(override_return_type(
    type_repr = "collections.abc.Callable[[str], typing.Any]",
    imports = ("collections.abc", "typing")
))]
pub fn get_callback(
    #[gen_stub(override_type(
        type_repr = "collections.abc.Callable[[int], str]",
        imports = ("collections.abc",)
    ))]
    cb: Bound<'_, PyAny>,
) -> PyResult<Bound<'_, PyAny>> {
    Ok(cb)
}
```

### Python Stub Syntax Override

```rust
#[gen_stub_pyfunction(python = r#"
    import collections.abc

    def get_callback(
        cb: collections.abc.Callable[[int], str]
    ) -> collections.abc.Callable[[str], typing.Any]:
        """Get callback function."""
"#)]
#[pyfunction]
pub fn get_callback(cb: Bound<'_, PyAny>) -> PyResult<Bound<'_, PyAny>> {
    Ok(cb)
}
```

## Import Management

### Automatic Imports

The stub generator automatically adds necessary imports:

```python
# If using typing.Sequence, typing.Mapping, etc.
import typing

# If using collections.abc types
import collections.abc

# If using numpy types (when numpy feature is enabled)
import numpy
```

### Custom Imports

For manual overrides, specify required imports:

```rust
#[gen_stub(override_type(
    type_repr = "MyCustomType",
    imports = ("my_package.submodule",)  // Adds: from my_package import submodule
))]
```

Import format:
- `("typing",)` → `import typing`
- `("collections.abc",)` → `import collections.abc`
- `("my_pkg.mod",)` → `from my_pkg import mod`

## Type Translation Pipeline

```
┌──────────────────────────────────────────────────────────────┐
│ 1. Rust Type (syn::Type)                                     │
│                                                              │
│    Example: Vec<Option<String>>                             │
└──────────────────────────────────────────────────────────────┘
                         ↓
┌──────────────────────────────────────────────────────────────┐
│ 2. Check for Manual Override                                │
│                                                              │
│    #[gen_stub(override_type(...))]                          │
│    or python parameter                                       │
└──────────────────────────────────────────────────────────────┘
                    Yes ↓    ↓ No
         ┌──────────────┘    └──────────────┐
         ↓                                    ↓
┌────────────────────┐         ┌─────────────────────────────┐
│ Use Override Type  │         │ 3. PyStubType Trait Lookup  │
│                    │         │                             │
│ TypeInfo::Override │         │ Vec<T>::type_input()        │
└────────────────────┘         └─────────────────────────────┘
         ↓                                    ↓
         │              ┌─────────────────────┘
         │              ↓
         │    ┌────────────────────────────────────┐
         │    │ 4. Recursive Type Construction     │
         │    │                                    │
         │    │ TypeInfo::with_generic(            │
         │    │     "typing.Sequence",             │
         │    │     vec![Option<String>::type_input()]
         │    │ )                                  │
         │    └────────────────────────────────────┘
         │              ↓
         │    ┌────────────────────────────────────┐
         │    │ 5. Nested Type Resolution          │
         │    │                                    │
         │    │ Option<String>::type_input()       │
         │    │   → str | None                     │
         │    └────────────────────────────────────┘
         │              ↓
         └─────────────→│
                        ↓
         ┌──────────────────────────────────────────┐
         │ 6. Final TypeInfo                        │
         │                                          │
         │ TypeInfo::GenericType {                  │
         │     base: "typing.Sequence",             │
         │     args: [TypeInfo::UnionType(          │
         │         [builtin("str"), none()]         │
         │     )]                                   │
         │ }                                        │
         └──────────────────────────────────────────┘
                        ↓
         ┌──────────────────────────────────────────┐
         │ 7. Python Type Syntax                    │
         │                                          │
         │ typing.Sequence[str | None]              │
         └──────────────────────────────────────────┘
```

## Special Cases

### Phantom Data

```rust
use std::marker::PhantomData;

struct MyType<T> {
    data: Vec<u8>,
    _phantom: PhantomData<T>,
}

impl<T> PyStubType for MyType<T> {
    fn type_input() -> TypeInfo {
        TypeInfo::class(None, "MyType")
        // T is not exposed to Python
    }

    fn type_output() -> TypeInfo {
        TypeInfo::class(None, "MyType")
    }
}
```

### Lifetime Erasure

All Rust lifetimes are erased in Python stubs:

```rust
// Rust: fn process<'a>(data: &'a str) -> &'a str
// Python: def process(data: str) -> str
```

### Unsupported Types

Types without `PyStubType` implementation fall back to `typing.Any`:

```rust
use std::sync::Arc;

#[pyfunction]
fn get_arc(x: Arc<i32>) -> Arc<i32> {
    x
}

// Generated stub:
// def get_arc(x: typing.Any) -> typing.Any: ...
```

## Testing Type Mappings

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pyo3_stub_gen::{PyStubType, TypeInfo};

    #[test]
    fn test_vec_type_mapping() {
        assert_eq!(
            Vec::<i32>::type_input().to_string(),
            "typing.Sequence[int]"
        );
        assert_eq!(
            Vec::<i32>::type_output().to_string(),
            "list[int]"
        );
    }

    #[test]
    fn test_optional_type() {
        assert_eq!(
            Option::<String>::type_input().to_string(),
            "str | None"
        );
    }
}
```

### Type Checker Validation

Generated stubs should pass type checkers:

```bash
# mypy
uv run mypy --strict your_module.pyi

# pyright
uv run pyright your_module.pyi

# stubtest
uv run stubtest your_module --ignore-missing-stub
```

## Module Locations

Type mapping implementations are organized as:

```
pyo3-stub-gen/src/stub_type/
├── builtins.rs        # Primitive types (int, str, bool, etc.)
├── collections.rs     # Vec, HashMap, tuples, etc.
├── pyo3.rs           # PyO3 types (Py, Bound, PyAny, etc.)
├── numpy.rs          # NumPy array types (feature-gated)
├── either.rs         # either::Either (feature-gated)
└── rust_decimal.rs   # Decimal type (feature-gated)
```

## Feature Flags

Type support can be conditionally enabled:

| Feature | Types Enabled | Requires |
|---------|---------------|----------|
| `numpy` | `PyArray*` types | `numpy` crate |
| `either` | `Either<L, R>` | `either` crate |
| `rust_decimal` | `Decimal` | `rust_decimal` crate |

Enable in `Cargo.toml`:

```toml
[dependencies]
pyo3-stub-gen = { version = "...", features = ["numpy", "either"] }
```

## Best Practices

### 1. Choose Appropriate Input/Output Types

```rust
// Good: Accept sequences, return list
impl PyStubType for MyVec {
    fn type_input() -> TypeInfo {
        TypeInfo::with_generic("typing.Sequence", vec![/* ... */])
    }
    fn type_output() -> TypeInfo {
        TypeInfo::with_generic("list", vec![/* ... */])
    }
}
```

### 2. Use Union Types for Flexibility

```rust
// Good: Accept multiple types
fn type_input() -> TypeInfo {
    TypeInfo::union(vec![
        TypeInfo::builtin("int"),
        TypeInfo::builtin("str"),
    ])
}
```

### 3. Document Custom Type Mappings

```rust
/// Maps to `MyPythonClass` in the generated stub.
///
/// Input: Accepts `MyPythonClass` instances
/// Output: Returns `MyPythonClass` instances
impl PyStubType for MyRustType {
    // ...
}
```

### 4. Test Type Mappings

Always validate generated stubs with type checkers to ensure correctness.

## Troubleshooting

### Issue: Type Not Found

**Error**: `the trait PyStubType is not implemented for CustomType`

**Solution**: Implement `PyStubType` for your custom type or use manual override.

### Issue: Wrong Python Type

**Problem**: Generated stub has incorrect type annotation.

**Solution**: Check `type_input()` vs `type_output()` implementation, or use manual override.

### Issue: Missing Imports

**Problem**: Generated stub references types without importing them.

**Solution**: Ensure `TypeInfo` construction includes necessary import information, or use `#[gen_stub(override_type(imports = (...)))]`.

## Related Documentation

- [Architecture](./architecture.md) - Overall system design
- [Default Value for Function Arguments](./default-value-arguments.md) - Function parameter defaults
- [Python Stub Syntax Support](./python-stub-syntax.md) - Manual type specification
