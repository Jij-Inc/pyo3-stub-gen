# Default Value for Class Members

## Overview

This feature enables pyo3-stub-gen to include default values for class attributes and properties in generated Python stub files (`.pyi`). Unlike function arguments, class member default values are specified using the `#[gen_stub(default = expr)]` attribute and are displayed differently in the generated stubs depending on the member type.

## Motivation

When defining Python classes in Rust using PyO3, class attributes often have default values that should be documented in the type stubs:

```rust
#[gen_stub_pyclass]
#[pyclass]
struct Config {
    #[pyo3(get, set)]
    #[gen_stub(default = Config::default().timeout)]
    timeout: u32,

    #[pyo3(get)]
    port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self { timeout: 30, port: 8080 }
    }
}
```

Without default value information, the stub lacks important documentation:

```python
# Without default value feature
class Config:
    @property
    def timeout(self) -> int: ...
    @timeout.setter
    def timeout(self, value: int) -> None: ...
    @property
    def port(self) -> int: ...
```

With this feature, users can see the default values:

```python
# With default value feature
class Config:
    @property
    def timeout(self) -> int:
        r"""
        ```python
        default = 30
        ```
        """
    @timeout.setter
    def timeout(self, value: int) -> None:
        r"""
        ```python
        default = 30
        ```
        """
    @property
    def port(self) -> int: ...
```

## Architecture

### Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ Compile-time (pyo3-stub-gen-derive)                            │
│                                                                 │
│  #[gen_stub(default = expr)]                                   │
│           ↓                                                     │
│  parse_gen_stub_default() → syn::Expr                         │
│           ↓                                                     │
│  MemberInfo::to_tokens()                                       │
│           ↓                                                     │
│  Generate: Some(|| fmt_py_obj(expr))                          │
└─────────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────────┐
│ Runtime (stub_gen binary)                                       │
│                                                                 │
│  Execute closure → Rust value                                   │
│           ↓                                                     │
│  fmt_py_obj() → Convert to Python representation               │
│           ↓                                                     │
│  String (e.g., "30", "'config.json'")                          │
└─────────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────────┐
│ Generation (pyo3-stub-gen)                                      │
│                                                                 │
│  MemberDef::fmt() / GetterDisplay / SetterDisplay              │
│           ↓                                                     │
│  Write to .pyi file                                            │
│           ↓                                                     │
│  - Class variable: VALUE: int = 42                             │
│  - Property: Embedded in docstring                             │
└─────────────────────────────────────────────────────────────────┘
```

## Data Structures

### MemberInfo Struct

**Location**: `pyo3-stub-gen/src/type_info.rs:127-137`

```rust
pub struct MemberInfo {
    pub name: &'static str,
    pub r#type: fn() -> TypeInfo,
    pub doc: &'static str,
    pub default: Option<fn() -> String>,  // Default value for attributes
    pub deprecated: Option<DeprecatedInfo>,
}
```

**Design rationale**:
- `Option<fn() -> String>`: Closure-based storage defers evaluation until stub generation
- Used for both getters and setters to document the initial/expected value
- Shared across `#[pyo3(get)]`, `#[pyo3(set)]`, and `#[classattr]`

### MemberDef Struct (Runtime)

**Location**: `pyo3-stub-gen/src/generate/member.rs:7-15`

```rust
pub struct MemberDef {
    pub name: &'static str,
    pub r#type: TypeInfo,
    pub doc: &'static str,
    pub default: Option<String>,  // Evaluated default value
    pub deprecated: Option<DeprecatedInfo>,
}
```

## Member Types

PyO3 supports several kinds of class members, each with different default value behavior:

### 1. Class Attributes (`#[classattr]`)

**Rust code**:
```rust
#[gen_stub_pymethods]
#[pymethods]
impl MyClass {
    #[classattr]
    const VERSION: &'static str = "1.0.0";
}
```

**Generated stub**:
```python
class MyClass:
    VERSION: str = '1.0.0'
```

**Note**: For `#[classattr]`, the default value is automatically extracted from the constant value. The `#[gen_stub(default = ...)]` attribute is optional.

### 2. Getters (`#[pyo3(get)]`)

**Rust code**:
```rust
#[gen_stub_pyclass]
#[pyclass]
struct MyClass {
    #[pyo3(get, set)]
    #[gen_stub(default = MyClass::default().value)]
    value: i32,
}

impl Default for MyClass {
    fn default() -> Self {
        Self { value: 42 }
    }
}
```

**Generated stub**:
```python
class MyClass:
    @property
    def value(self) -> int:
        r"""
        ```python
        default = 42
        ```
        """
```

### 3. Setters (`#[pyo3(set)]`)

When both getter and setter exist with a default value:

**Generated stub**:
```python
class MyClass:
    @property
    def value(self) -> int:
        r"""
        ```python
        default = 42
        ```
        """
    @value.setter
    def value(self, value: int) -> None:
        r"""
        ```python
        default = 42
        ```
        """
```

### 4. Custom Getters/Setters (Methods)

**Rust code**:
```rust
#[gen_stub_pymethods]
#[pymethods]
impl MyClass {
    #[getter]
    #[gen_stub(default = MyClass::default().y)]
    pub fn get_y(&self) -> usize {
        self.y
    }
}
```

**Generated stub**:
```python
class MyClass:
    @property
    def y(self) -> int:
        r"""
        ```python
        default = 10
        ```
        """
```

## Implementation Details

### 1. Attribute Parsing

**File**: `pyo3-stub-gen-derive/src/gen_stub/attr.rs:320-326`

```rust
pub fn parse_gen_stub_default(attrs: &[Attribute]) -> Result<Option<Expr>> {
    for attr in parse_gen_stub_attrs(attrs, AttributeLocation::Function, None)? {
        if let StubGenAttr::Default(default) = attr {
            return Ok(Some(default));
        }
    }
    Ok(None)
}
```

The parser extracts default values from:
- Field attributes: `#[gen_stub(default = expr)]`
- Method attributes: `#[gen_stub(default = expr)]` on getters/setters

### 2. Code Generation for Fields

**File**: `pyo3-stub-gen-derive/src/gen_stub/member.rs:179-189`

```rust
impl TryFrom<&Field> for MemberInfo {
    fn try_from(field: &Field) -> Result<Self> {
        let attrs = &field.attrs;
        // ... parse type and documentation ...
        let default = parse_gen_stub_default(&attrs)?;
        let deprecated = extract_deprecated(&attrs);

        Ok(Self {
            name,
            r#type: TypeOrOverride::RustType { r#type: ty },
            doc,
            default,  // Captured from attribute
            deprecated,
        })
    }
}
```

### 3. Code Generation for Methods (Getters/Setters)

**File**: `pyo3-stub-gen-derive/src/gen_stub/member.rs:53-72`

```rust
impl MemberInfo {
    pub fn getter(item: &ImplItemFn) -> Result<Option<Self>> {
        let default = parse_gen_stub_default(attrs)?;
        // ...
        Ok(Some(Self {
            name,
            r#type: extract_return_type(&sig.output, attrs)?,
            default,  // Captured from method attribute
            deprecated: extract_deprecated(attrs),
        }))
    }
}
```

### 4. Closure Generation

**File**: `pyo3-stub-gen-derive/src/gen_stub/member.rs:202-225`

```rust
let default = default
    .as_ref()
    .map(|value| {
        match r#type {
            TypeOrOverride::RustType { r#type: ty }
            | TypeOrOverride::OverrideType { r#type: ty, .. } => {
                quote! {
                    let v: #ty = #value;
                    ::pyo3_stub_gen::util::fmt_py_obj(v)
                }
            }
        }
    })
    .map_or(quote! {None}, |default| {
        quote! {Some({
            fn _fmt() -> String {
                #default
            }
            _fmt
        })}
    });
```

**Features**:
- Type-checked conversion: `let v: #ty = #value`
- Uses `fmt_py_obj` for Python representation
- Wrapped in closure for deferred evaluation

### 5. Stub File Generation

#### For Class Attributes (Constants)

**File**: `pyo3-stub-gen/src/generate/member.rs:41-61`

```rust
impl fmt::Display for MemberDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent = indent();
        write!(f, "{indent}{}: {}", self.name, self.r#type)?;
        if let Some(default) = &self.default {
            write!(f, " = {default}")?;  // Inline default value
        }
        writeln!(f)?;
        docstring::write_docstring(f, self.doc, indent)?;
        Ok(())
    }
}
```

**Output**: `CONSTANT: int = 42`

#### For Properties (Getters)

**File**: `pyo3-stub-gen/src/generate/member.rs:66-98`

```rust
impl fmt::Display for GetterDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // ... write @property decorator ...
        let doc = if let Some(default) = &self.0.default {
            if default == "..." {
                Cow::Borrowed(self.0.doc)
            } else {
                Cow::Owned(format!(
                    "{}\n```python\ndefault = {default}\n```",
                    self.0.doc
                ))
            }
        } else {
            Cow::Borrowed(self.0.doc)
        };
        // ... write docstring ...
    }
}
```

**Output**:
```python
@property
def field(self) -> int:
    r"""
    Field documentation
    ```python
    default = 42
    ```
    """
```

#### For Properties (Setters)

**File**: `pyo3-stub-gen/src/generate/member.rs:100-133`

Similar to getters, setters also embed default values in their docstrings.

## Usage Examples

### Simple Field with Default

```rust
#[gen_stub_pyclass]
#[pyclass]
struct A {
    #[gen_stub(default = A::default().x)]
    #[pyo3(get, set)]
    x: usize,
}

impl Default for A {
    fn default() -> Self {
        Self { x: 2 }
    }
}
```

**Generated stub**:
```python
class A:
    @property
    def x(self) -> int:
        r"""
        ```python
        default = 2
        ```
        """
    @x.setter
    def x(self, value: int) -> None:
        r"""
        ```python
        default = 2
        ```
        """
```

### Class Attribute (Constant)

```rust
#[gen_stub_pymethods]
#[pymethods]
impl MyClass {
    #[classattr]
    const MAX_SIZE: usize = 1024;
}
```

**Generated stub**:
```python
class MyClass:
    MAX_SIZE: int = 1024
```

### Custom Getter with Default

```rust
#[gen_stub_pymethods]
#[pymethods]
impl A {
    #[getter]
    #[gen_stub(default = A::default().y)]
    pub fn get_y(&self) -> usize {
        self.y
    }
}
```

**Generated stub**:
```python
class A:
    @property
    def y(self) -> int:
        r"""
        ```python
        default = 10
        ```
        """
```

### Complex Default Values

```rust
#[gen_stub_pyclass]
#[pyclass]
struct Config {
    #[pyo3(get, set)]
    #[gen_stub(default = vec!["localhost".to_string()])]
    hosts: Vec<String>,
}
```

**Generated stub**:
```python
class Config:
    @property
    def hosts(self) -> list[str]:
        r"""
        ```python
        default = ['localhost']
        ```
        """
```

## Design Decisions

### 1. Docstring Embedding for Properties

**Decision**: Embed default values in property docstrings rather than the signature.

**Rationale**:
- Python properties don't support default values in their signatures:
  ```python
  @property
  def field(self) -> int = 42:  # ❌ Syntax error
  ```
- Docstring placement provides visible documentation in IDEs
- Code block formatting makes it clear this is a default value
- Preserves existing docstring content

### 2. Inline Values for Class Attributes

**Decision**: Show default values inline for class attributes (constants).

**Rationale**:
- Matches standard Python stub conventions
- Class-level assignments are valid Python syntax:
  ```python
  class MyClass:
      CONSTANT: int = 42  # ✅ Valid
  ```
- Provides better IDE support (hover shows value)
- Consistent with how Python type checkers expect class attributes

### 3. Closure-based Storage

**Decision**: Use `Option<fn() -> String>` instead of `Option<String>`.

**Rationale**:
- Allows evaluation of complex expressions (e.g., `A::default().x`)
- Defers execution until stub generation runtime
- Enables access to runtime-initialized values
- Consistent with parameter default value implementation

### 4. Special Handling for "..." Fallback

**Decision**: Don't display "..." in docstrings when conversion fails.

**Rationale**:
- "..." is a placeholder indicating conversion failure
- Showing it in documentation could confuse users
- Better to omit default information than show incorrect/unclear values

### 5. Same Default for Getter and Setter

**Decision**: Apply the same default value to both getter and setter.

**Rationale**:
- Represents the initial/expected value of the field
- Helps users understand what value the field starts with
- Consistent documentation across both accessors

## Special Cases

### 1. Read-only Properties

For fields with only `#[pyo3(get)]`:

```rust
#[gen_stub_pyclass]
#[pyclass]
struct MyClass {
    #[pyo3(get)]
    #[gen_stub(default = 42)]
    readonly: i32,
}
```

**Generated stub**:
```python
class MyClass:
    @property
    def readonly(self) -> int:
        r"""
        ```python
        default = 42
        ```
        """
```

### 2. Write-only Properties

For fields with only `#[pyo3(set)]`:

```rust
#[gen_stub_pyclass]
#[pyclass]
struct MyClass {
    #[pyo3(set)]
    #[gen_stub(default = "secret")]
    writeonly: String,
}
```

**Generated stub**:
```python
class MyClass:
    @writeonly.setter
    def writeonly(self, value: str) -> None:
        r"""
        ```python
        default = 'secret'
        ```
        """
```

### 3. Deprecated Members

**Rust code**:
```rust
#[gen_stub_pymethods]
#[pymethods]
impl MyClass {
    #[deprecated(since = "1.0.0", note = "Use new_field instead")]
    #[classattr]
    const OLD_CONSTANT: i32 = 100;
}
```

**Generated stub**:
```python
class MyClass:
    OLD_CONSTANT: int = 100
```

**Note**: Python constants cannot have decorators, so deprecation information is logged as a warning but not included in the stub.

## Limitations

### 1. No Default Values in Property Signatures

Unlike function parameters, Python properties cannot have default values in their type signatures. This is a Python language limitation, not a pyo3-stub-gen limitation.

### 2. Feature Dependency

Like argument defaults, member default conversion requires the `infer_signature` feature:

```toml
[dependencies]
pyo3-stub-gen = { version = "...", features = ["infer_signature"] }
```

### 3. Type Representability

Same limitations as argument defaults apply:
- Only types with valid `repr()` implementations are supported
- Custom types fall back to `"..."` and are not displayed

### 4. Constants Cannot Be Deprecated

Python syntax doesn't allow decorators on class-level variable assignments:

```python
class MyClass:
    @deprecated("message")  # ❌ Syntax error
    CONSTANT: int = 42
```

If deprecation is needed, consider using a function or property instead.

### 5. Evaluation Context

Default value expressions must be evaluable in the context where the `MemberInfo` is created (typically in procedural macro expansion). Values depending on runtime state may not work as expected.

## Testing

Member default value functionality is tested through:

1. **Integration tests** in example projects
2. **Snapshot tests** for getter/setter generation
3. **Type checker validation** (mypy, pyright, stubtest)

Example test locations:
- `examples/pure/src/lib.rs:66-78` - Field defaults
- `examples/feature_gated/src/gen_stub_default.rs` - Getter method defaults
- `pyo3-stub-gen/src/generate/member.rs` - Display implementation tests

## Related Documentation

- [Default Value for Function Arguments](./default-value-arguments.md) - Default values for function/method parameters
- PyO3 documentation: [Class Attributes](https://pyo3.rs/latest/class.html#class-attributes)
