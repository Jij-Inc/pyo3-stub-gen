# Default Value for Function Arguments

## Overview

This feature enables pyo3-stub-gen to include default values for function and method parameters in generated Python stub files (`.pyi`). There are two approaches to specifying default values:

1. **Rust-based approach**: Use PyO3's `#[pyo3(signature = ...)]` attribute with Rust expressions
2. **Python-based approach**: Write Python stub syntax directly using `python` parameter or macros

Both approaches produce the same result in the generated `.pyi` files, but differ in how default values are specified and processed.

## Two Approaches to Specify Default Values

### Approach 1: Rust-based (PyO3 Signature)

Use PyO3's `#[pyo3(signature = ...)]` attribute with **Rust expressions**:

```rust
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (x, y = 10, z = "hello"))]
fn my_function(x: i32, y: i32, z: String) -> i32 {
    x + y
}
```

**Default values are Rust expressions**: `10`, `"hello"`, `vec![1, 2]`, `MyEnum::Variant`, etc.

### Approach 2: Python-based (Direct Stub Syntax)

Write Python stub syntax directly with **Python expressions**:

```rust
#[gen_stub_pyfunction(python = r#"
    def my_function(x: int, y: int = 10, z: str = 'hello') -> int:
        """My function."""
"#)]
#[pyfunction]
fn my_function(x: PyObject, y: PyObject, z: PyObject) -> i32 {
    // Implementation
}
```

**Default values are Python expressions**: `10`, `'hello'`, `[1, 2]`, `MyEnum.VARIANT`, etc.

### Comparison

| Aspect | Rust-based | Python-based |
|--------|-----------|-------------|
| **Syntax** | `#[pyo3(signature = (x = expr))]` | `#[gen_stub_pyfunction(python = r#"def foo(x = expr): ..."#)]` |
| **Expression language** | Rust | Python |
| **Type checking** | Compile-time (Rust) | Parse-time (Python AST) |
| **Conversion** | `fmt_py_obj(rust_value)` → Python repr | Direct use (already Python) |
| **Best for** | PyO3 functions with Rust types | Complex types, overloads, type overrides |
| **Runtime behavior** | Defined by PyO3 signature | Defined by Rust implementation |

### When to Use Which?

**Use Rust-based approach when**:
- Writing standard PyO3 functions with Rust types
- Want compile-time type checking of default values
- Default values can be expressed in Rust

**Use Python-based approach when**:
- Need complex Python types (e.g., `collections.abc.Callable`)
- Implementing overloaded functions
- Want to match Python conventions exactly
- Need type overrides that don't map cleanly from Rust

Both approaches produce the same output:

```python
def my_function(x: int, y: int = 10, z: str = 'hello') -> int: ...
```

## Architecture

### Data Flow: Rust-based Approach

```
┌─────────────────────────────────────────────────────────────────┐
│ Compile-time (pyo3-stub-gen-derive)                            │
│                                                                 │
│  #[pyo3(signature = (x = RUST_EXPR))]                         │
│           ↓                                                     │
│  parse_signature() → Extract Rust default expressions          │
│           ↓                                                     │
│  ParameterWithKind::to_tokens()                                │
│           ↓                                                     │
│  Generate: ParameterDefault::Expr(|| fmt_py_obj(RUST_EXPR))   │
└─────────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────────┐
│ Runtime (stub_gen binary)                                       │
│                                                                 │
│  Execute closure → Rust value                                   │
│           ↓                                                     │
│  fmt_py_obj() → Convert to Python representation               │
│           ↓                                                     │
│  String (e.g., "10", "'hello'", "Number.FLOAT")                │
└─────────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────────┐
│ Generation (pyo3-stub-gen)                                      │
│                                                                 │
│  Parameter::fmt()                                              │
│           ↓                                                     │
│  Write to .pyi file                                            │
│           ↓                                                     │
│  def foo(x: int = 10) -> None: ...                            │
└─────────────────────────────────────────────────────────────────┘
```

### Data Flow: Python-based Approach

```
┌─────────────────────────────────────────────────────────────────┐
│ Compile-time (pyo3-stub-gen-derive)                            │
│                                                                 │
│  #[gen_stub_pyfunction(python = r#"def foo(x = PY_EXPR): ..."#)]│
│           ↓                                                     │
│  parse_python_stub() → rustpython-parser                       │
│           ↓                                                     │
│  extract_parameters() → Python AST                             │
│           ↓                                                     │
│  python_expr_to_syn_expr() → Convert Python AST to syn::Expr   │
│           ↓                                                     │
│  Generate: ParameterDefault::Expr(|| PY_EXPR_STR.to_string()) │
└─────────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────────┐
│ Runtime (stub_gen binary)                                       │
│                                                                 │
│  Execute closure → Python expression string                     │
│           ↓                                                     │
│  String (e.g., "10", "'hello'", "Number.FLOAT")                │
│  (No fmt_py_obj conversion - already Python syntax)            │
└─────────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────────┐
│ Generation (pyo3-stub-gen)                                      │
│                                                                 │
│  Parameter::fmt()                                              │
│           ↓                                                     │
│  Write to .pyi file                                            │
│           ↓                                                     │
│  def foo(x: int = 10) -> None: ...                            │
└─────────────────────────────────────────────────────────────────┘
```

**Key difference**: Rust-based approach converts Rust values to Python representation at runtime, while Python-based approach uses Python expressions directly.

## Data Structures

### ParameterDefault Enum

**Location**: `pyo3-stub-gen/src/type_info.rs:67-74`

```rust
/// Default value of a parameter
#[derive(Debug, Clone)]
pub enum ParameterDefault {
    /// No default value
    None,
    /// Default value expression as a string
    Expr(fn() -> String),
}
```

**Design rationale**:
- `None`: Represents parameters without defaults (positional-only or required parameters)
- `Expr(fn() -> String)`: Closure that evaluates the default value at stub generation time
  - Defers evaluation to runtime, allowing complex expressions
  - Returns Python representation as a string

### ParameterInfo Struct

**Location**: `pyo3-stub-gen/src/type_info.rs:91-105`

```rust
pub struct ParameterInfo {
    pub name: &'static str,
    pub kind: ParameterKind,
    pub type_info: fn() -> TypeInfo,
    pub default: ParameterDefault,  // Default value field
}
```

### Parameter Struct (Runtime)

**Location**: `pyo3-stub-gen/src/generate/parameters.rs:17-30`

```rust
pub struct Parameter {
    pub name: &'static str,
    pub kind: ParameterKind,
    pub type_info: TypeInfo,
    pub default: ParameterDefault,  // Evaluated default value
}
```

## Implementation Details

The implementation differs significantly between the two approaches.

### Rust-based Approach

#### 1. Signature Parsing

**File**: `pyo3-stub-gen-derive/src/gen_stub/parameter.rs:138-327`

The `Parameters::try_from` implementation parses PyO3's `#[pyo3(signature = ...)]` attribute:

```rust
// Extract signature from #[pyo3(signature = (...))] attribute
for attr in parse_pyo3_attrs(&item_fn.attrs)? {
    if let Attr::Signature(signature) = attr {
        // Parse function signature parameters
        for pair in signature.pairs() {
            match pair.value() {
                // Handle default values: param = value
                FnArg::PosArgWithDefault(arg, _, value) => {
                    parameters.push(ParameterWithKind {
                        arg_info,
                        kind,
                        default_expr: Some(value.clone()),  // Capture default expression
                    });
                }
                // Handle parameters without defaults
                FnArg::PosArg(arg) => {
                    parameters.push(ParameterWithKind {
                        arg_info,
                        kind,
                        default_expr: None,  // No default
                    });
                }
            }
        }
    }
}
```

#### 2. Code Generation

**File**: `pyo3-stub-gen-derive/src/gen_stub/parameter.rs:22-102`

The `ToTokens` implementation generates code to capture default values:

##### Case 1: RustType (Type-safe conversion)

```rust
TypeOrOverride::RustType { r#type } => {
    let default = if value.to_token_stream().to_string() == "None" {
        // Special handling for None literal
        quote! { "None".to_string() }
    } else {
        // Type-checked conversion
        quote! {
            let v: #r#type = #value;
            ::pyo3_stub_gen::util::fmt_py_obj(v)
        }
    };
    quote! {
        ::pyo3_stub_gen::type_info::ParameterDefault::Expr({
            fn _fmt() -> String {
                #default
            }
            _fmt
        })
    }
}
```

**Features**:
- Compile-time type checking: `let v: #r#type = #value`
- Runtime conversion via `fmt_py_obj` for proper Python representation
- Special case for `None` to avoid type inference issues

##### Case 2: OverrideType (Direct string conversion)

```rust
TypeOrOverride::OverrideType { .. } => {
    let mut value_str = value.to_token_stream().to_string();
    // Convert Rust bool literals to Python bool literals
    if value_str == "false" {
        value_str = "False".to_string();
    } else if value_str == "true" {
        value_str = "True".to_string();
    }
    quote! {
        ::pyo3_stub_gen::type_info::ParameterDefault::Expr({
            fn _fmt() -> String {
                #value_str.to_string()
            }
            _fmt
        })
    }
}
```

**Features**:
- Direct string conversion (no runtime type checking)
- Boolean literal translation for Python compatibility
- Used when type override is explicitly specified

#### 3. Runtime Value Conversion

**File**: `pyo3-stub-gen/src/util.rs:57-75`

```rust
pub fn fmt_py_obj<T: for<'py> pyo3::IntoPyObjectExt<'py>>(obj: T) -> String {
    #[cfg(feature = "infer_signature")]
    {
        pyo3::Python::initialize();
        pyo3::Python::attach(|py| -> String {
            if let Ok(any) = obj.into_bound_py_any(py) {
                if all_builtin_types(&any) || valid_external_repr(&any).is_some_and(|valid| valid) {
                    if let Ok(py_str) = any.repr() {
                        return py_str.to_string();
                    }
                }
            }
            "...".to_owned()
        })
    }
    #[cfg(not(feature = "infer_signature"))]
    {
        "...".to_owned()
    }
}
```

**Conversion examples**:

| Rust Value | Python Representation |
|------------|----------------------|
| `42` | `"42"` |
| `3.14` | `"3.14"` |
| `true` / `false` | `"True"` / `"False"` |
| `"hello"` | `"'hello'"` |
| `vec![1, 2, 3]` | `"[1, 2, 3]"` |
| `Some(10)` | `"10"` |
| `None::<i32>` | `"None"` |
| `Number::Float` | `"Number.FLOAT"` (enum) |
| Custom types | `"..."` (fallback) |

### Python-based Approach

#### 1. Python Stub Parsing

**File**: `pyo3-stub-gen-derive/src/gen_stub/parse_python/pyfunction.rs`

Python stub syntax is parsed using `rustpython-parser`:

```rust
// Parse Python stub string
let module = parse_program(&stub_str, "<embedded>")
    .map_err(|e| syn::Error::new(proc_macro2::Span::call_site(), format!("Failed to parse Python: {}", e)))?;

// Extract function definition from AST
for stmt in &module.statements {
    if let ast::Stmt::FunctionDef(func_def) = stmt {
        // Extract parameters with default values
        process_function_parameters(&func_def.args)?;
    }
}
```

#### 2. Parameter Extraction with Defaults

**File**: `pyo3-stub-gen-derive/src/gen_stub/parse_python.rs:104-196`

Default values are extracted from Python AST:

```rust
let process_arg_with_default = |arg: &ast::ArgWithDefault, kind: ParameterKind| -> Result<Option<ParameterWithKind>> {
    let arg_name = arg.def.arg.to_string();

    // Convert default value from Python AST to syn::Expr
    let default_expr = if let Some(default) = &arg.default {
        Some(python_expr_to_syn_expr(default)?)
    } else {
        None
    };

    Ok(Some(ParameterWithKind {
        arg_info,
        kind,
        default_expr,  // Python expression converted to Rust syn::Expr
    }))
};
```

#### 3. Python Expression to Rust Conversion

**File**: `pyo3-stub-gen-derive/src/gen_stub/parse_python.rs:290-348`

```rust
fn python_expr_to_syn_expr(expr: &ast::Expr) -> Result<syn::Expr> {
    let expr_str = match expr {
        // Python literals
        ast::Expr::Constant(constant) => match &constant.value {
            ast::Constant::None => "None".to_string(),
            ast::Constant::Bool(b) => b.to_string(),  // Python True/False
            ast::Constant::Int(i) => i.to_string(),
            ast::Constant::Float(f) => f.to_string(),
            ast::Constant::Str(s) => format!("\"{}\"", s.escape_default()),
            ast::Constant::Ellipsis => "...".to_string(),
            _ => return Err(...),
        },
        // Python attribute access: MyEnum.VARIANT
        ast::Expr::Attribute(_) => expr_to_type_string(expr)?,
        // Python unary operations: -42
        ast::Expr::UnaryOp(unary) => {
            if matches!(unary.op, ast::UnaryOp::USub) {
                format!("-{}", /* operand */)
            } else {
                "...".to_string()
            }
        }
        // Complex types fall back to "..."
        ast::Expr::List(_) | ast::Expr::Tuple(_) | ast::Expr::Dict(_) => "...".to_string(),
        _ => "...".to_string(),
    };

    // Parse as Rust syn::Expr
    syn::parse_str(&expr_str).map_err(|e| ...)
}
```

**Key conversions**:
- Python `True`/`False` → Rust `true`/`false` (string form, later converted back)
- Python `None` → Rust `None` (string)
- Python `Number.FLOAT` → Rust `Number::FLOAT` (attribute access converted to path)
- Complex Python expressions → `"..."` (placeholder)

#### 4. Code Generation

Once converted to `syn::Expr`, the Python-based approach follows the same code generation path as the Rust-based approach, but with `OverrideType` handling:

```rust
// For Python-based parameters, typically OverrideType is used
TypeOrOverride::OverrideType { .. } => {
    let mut value_str = value.to_token_stream().to_string();
    // Boolean conversion is already done in python_expr_to_syn_expr
    // String is used directly as Python expression
    quote! {
        ::pyo3_stub_gen::type_info::ParameterDefault::Expr({
            fn _fmt() -> String {
                #value_str.to_string()
            }
            _fmt
        })
    }
}
```

**No `fmt_py_obj` conversion**: The expression is already in Python syntax.

### Common Path: Stub File Generation

**File**: `pyo3-stub-gen/src/generate/parameters.rs:52-70`

```rust
impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ParameterKind::VarPositional => {
                write!(f, "*{}: {}", self.name, self.type_info)
            }
            ParameterKind::VarKeyword => {
                write!(f, "**{}: {}", self.name, self.type_info)
            }
            _ => {
                write!(f, "{}: {}", self.name, self.type_info)?;
                match &self.default {
                    ParameterDefault::None => Ok(()),
                    ParameterDefault::Expr(expr) => write!(f, " = {}", expr),
                }
            }
        }
    }
}
```

**Output format**: `param_name: Type = default_value`

Both Rust-based and Python-based approaches converge at this point, producing identical stub output.

## Usage Examples

All examples below show both approaches that produce the same output.

### Basic Types

**Rust-based**:
```rust
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (x, y = 10, z = 3.14))]
fn basic_defaults(x: i32, y: i32, z: f64) -> f64 {
    x as f64 + y as f64 + z
}
```

**Python-based**:
```rust
#[gen_stub_pyfunction(python = r#"
    def basic_defaults(x: int, y: int = 10, z: float = 3.14) -> float:
        """Basic defaults example."""
"#)]
#[pyfunction]
fn basic_defaults(x: PyObject, y: PyObject, z: PyObject) -> f64 {
    // Implementation
}
```

**Generated stub**:
```python
def basic_defaults(x: int, y: int = 10, z: float = 3.14) -> float: ...
```

### String Defaults

**Rust-based**:
```rust
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (name = "World"))]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
```

**Python-based**:
```rust
#[gen_stub_pyfunction(python = r#"
    def greet(name: str = 'World') -> str: ...
"#)]
#[pyfunction]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
```

**Generated stub**:
```python
def greet(name: str = 'World') -> str: ...
```

### Boolean Defaults

```rust
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (verbose = false))]
fn process(verbose: bool) -> () {
    // ...
}
```

**Generated stub**:
```python
def process(verbose: bool = False) -> None: ...
```

### Collection Defaults

```rust
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (items = vec![1, 2, 3]))]
fn process_items(items: Vec<i32>) -> Vec<i32> {
    items
}
```

**Generated stub**:
```python
def process_items(items: list[int] = [1, 2, 3]) -> list[int]: ...
```

### Enum Defaults

**Rust-based**:
```rust
#[gen_stub_pyclass_enum]
#[pyclass(eq, eq_int)]
#[derive(Clone, PartialEq)]
pub enum Number {
    Float,
    Integer,
}

#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (num = Number::Float))]
fn default_value(num: Number) -> Number {
    num
}
```

**Python-based**:
```rust
use pyo3_stub_gen::inventory::submit;

submit! {
    gen_function_from_python! {
        r#"
        def default_value(num: Number = Number.FLOAT) -> Number: ...
        "#
    }
}
```

**Generated stub**:
```python
def default_value(num: Number = Number.FLOAT) -> Number: ...
```

### Optional Defaults

```rust
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (value = None))]
fn optional_arg(value: Option<i32>) -> Option<i32> {
    value
}
```

**Generated stub**:
```python
def optional_arg(value: int | None = None) -> int | None: ...
```

### Keyword-Only Parameters with Defaults

```rust
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (x, *, y = 5, z = "default"))]
fn keyword_only(x: i32, y: i32, z: String) -> i32 {
    x + y
}
```

**Generated stub**:
```python
def keyword_only(x: int, *, y: int = 5, z: str = 'default') -> int: ...
```

## Design Decisions

### 1. Two Approaches for Specifying Defaults

**Decision**: Provide both Rust-based (`#[pyo3(signature = ...)]`) and Python-based (`python = r#"..."#`) approaches.

**Rationale**:
- **Rust-based approach**:
  - Leverages existing PyO3 syntax developers already use
  - Provides compile-time type checking of default values
  - Single source of truth for both runtime and stub generation
  - Works seamlessly with Rust's type system
- **Python-based approach**:
  - Necessary for complex Python types that don't map from Rust
  - Essential for function overloading (multiple signatures for same function)
  - Allows expressing Python-specific conventions
  - Provides escape hatch when automatic conversion is insufficient
- **Having both**:
  - Each approach has distinct use cases
  - No single approach covers all scenarios
  - Users can choose based on their needs
  - Both produce identical stub output

### 2. Closure-based Storage

**Decision**: Use `fn() -> String` instead of storing the string directly.

**Rationale**:
- Defers evaluation until stub generation runtime
- Allows PyO3 initialization to complete before conversion
- Enables use of complex Rust expressions
- Keeps compile-time overhead minimal

### 3. Integration with PyO3's Signature Attribute

**Decision**: For Rust-based approach, extract defaults from `#[pyo3(signature = ...)]` rather than introducing a separate attribute.

**Rationale**:
- Single source of truth for function signatures
- Avoids duplication and potential inconsistencies
- Leverages existing PyO3 syntax that developers already know
- Ensures runtime behavior matches stub declarations
- PyO3's signature already defines runtime behavior

### 4. Python Representation via `repr()` (Rust-based)

**Decision**: Use Python's `repr()` function to convert values.

**Rationale**:
- Guarantees Python-compatible syntax
- Handles edge cases correctly (string escaping, quotes, etc.)
- Consistent with Python's own representation rules
- Falls back gracefully for non-representable types

### 5. Direct String Use for Python-based Approach

**Decision**: For Python-based approach, use Python expressions directly as strings without `fmt_py_obj` conversion.

**Rationale**:
- Python expressions are already in correct syntax
- No need for runtime conversion overhead
- Preserves exact Python syntax as written by developer
- Avoids potential conversion artifacts
- More predictable output matches input exactly

### 6. Python AST to Rust `syn::Expr` Conversion

**Decision**: Convert Python AST to Rust `syn::Expr` as an intermediate representation.

**Rationale**:
- Unifies both approaches in the code generation phase
- Allows reuse of existing `ParameterWithKind::to_tokens` implementation
- Provides compile-time validation that expressions are well-formed
- Simplifies the implementation by having a common representation
- Fallback to `"..."` for unsupported Python expressions provides graceful degradation

### 7. Type-safe Conversion Path (Rust-based)

**Decision**: Type-check default values at compile time when possible.

**Rationale**:
- Catches type mismatches early (e.g., `x: i32` with default `"string"`)
- Provides better error messages
- Ensures generated stubs match runtime behavior
- Minimal runtime overhead

### 8. Special Handling for None

**Decision**: Special case for `None` literal to avoid type inference issues.

**Rationale**:
- `None` in Rust doesn't have a concrete type without context
- Direct string conversion is simpler and safer
- Avoids needing to specify type parameters for `Option::None`

## Parameter Kinds and Defaults

Python supports several parameter kinds, each with different default value rules:

| Parameter Kind | Can Have Default? | Example |
|----------------|-------------------|---------|
| Positional-only (before `/`) | ✅ Yes | `def f(x=1, /): ...` |
| Positional-or-keyword | ✅ Yes | `def f(x=1): ...` |
| Var-positional (`*args`) | ❌ No | `def f(*args): ...` |
| Keyword-only (after `*`) | ✅ Yes | `def f(*, x=1): ...` |
| Var-keyword (`**kwargs`) | ❌ No | `def f(**kwargs): ...` |

**Note**: Once a parameter has a default value, all following parameters (except `*args` and `**kwargs`) must also have defaults.

## Limitations

### 1. Feature Dependency

Default value conversion requires the `infer_signature` feature to be enabled:

```toml
[dependencies]
pyo3-stub-gen = { version = "...", features = ["infer_signature"] }
```

Without this feature, all complex defaults fall back to `"..."`.

### 2. Type Representability

Only types that implement PyO3's `IntoPyObjectExt` trait and have valid `repr()` implementations are supported:

**Supported**:
- Primitive types: `i32`, `f64`, `bool`, `String`, etc.
- Collections: `Vec<T>`, `HashMap<K, V>`, `Option<T>`, etc. (where T, K, V are supported)
- PyO3 enums

**Not supported** (falls back to `"..."`):
- Custom structs without PyO3 bindings
- Complex Rust types (e.g., `Arc<Mutex<T>>`)
- Types with non-representable `repr()`

### 3. Evaluation Time

Default value expressions are evaluated during stub generation, not at Python import time:

```rust
// This counter increments during stub generation, not when Python imports the module
static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[pyfunction]
#[pyo3(signature = (x = COUNTER.fetch_add(1, Ordering::Relaxed)))]
fn with_counter(x: usize) -> usize { x }
```

Generated stub will have a fixed value, not a counter.

### 4. Mutable Defaults

Like Python, using mutable defaults in PyO3 creates shared state:

```rust
#[pyfunction]
#[pyo3(signature = (items = vec![]))]
fn append_item(items: Vec<i32>) -> Vec<i32> {
    // All calls share the same default vector!
    items
}
```

**Best practice**: Use `None` as default and create mutable objects inside the function.

## Testing

Default value functionality for arguments is tested through:

1. **Unit tests** in `pyo3-stub-gen/src/util.rs` for `fmt_py_obj` conversions
2. **Integration tests** in example projects
3. **Snapshot tests** in `pyo3-stub-gen-derive` for parameter parsing
4. **Type checker validation** using mypy, pyright, and stubtest

Example test locations:
- `examples/pure/src/lib.rs:336` - Enum default value
- `pyo3-stub-gen/src/util.rs:88-149` - Conversion tests
- `pyo3-stub-gen-derive/src/gen_stub/parse_python/pyfunction.rs` - Signature parsing tests

## Approach Comparison Summary

| Feature | Rust-based | Python-based |
|---------|-----------|-------------|
| **Attribute** | `#[pyo3(signature = (x = expr))]` | `#[gen_stub_pyfunction(python = r#"..."#)]` |
| **Expression language** | Rust | Python |
| **Compile-time checking** | ✅ Full type checking | ⚠️ Syntax checking only |
| **Runtime conversion** | `fmt_py_obj()` | Direct string use |
| **Feature dependency** | `infer_signature` for complex types | None |
| **Type mapping** | Automatic Rust→Python | Manual specification |
| **Overloading support** | ❌ No | ✅ Yes (via `gen_function_from_python!`) |
| **Complex Python types** | ⚠️ Limited | ✅ Full support |
| **Runtime behavior source** | PyO3 signature | Rust implementation |
| **Best for** | Standard PyO3 functions | Overloads, complex types |

**Recommendation**: Use Rust-based approach by default. Switch to Python-based when you need:
- Function overloading
- Complex Python types (e.g., `Callable`, `Protocol`)
- Type overrides that don't map from Rust
- Exact control over stub syntax

## Related Documentation

- [Default Value for Class Members](./default-value-members.md) - Default values for class attributes and properties
- PyO3 documentation: [Function Signatures](https://pyo3.rs/latest/function/signature.html)
- CLAUDE.md: Python Stub Syntax Support - Details on using `gen_function_from_python!` and related macros
