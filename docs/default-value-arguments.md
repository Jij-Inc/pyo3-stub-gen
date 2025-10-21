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
| **Intermediate form** | `DefaultExpr::Rust(syn::Expr)` | `DefaultExpr::Python(String)` |
| **Type checking** | Compile-time (Rust) | Parse-time (Python AST) |
| **Conversion timing** | Runtime (`fmt_py_obj`) | Compile-time (`python_ast_to_python_string`) |
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
│  build_parameters_from_ast() → Extract parameters from AST     │
│           ↓                                                     │
│  python_ast_to_python_string() → Convert Python AST to String  │
│           ↓                                                     │
│  DefaultExpr::Python(python_string)                            │
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

**Key difference**: Rust-based approach converts Rust values to Python representation at runtime via `fmt_py_obj`, while Python-based approach converts Python AST to Python string at compile-time and uses it directly.

## Data Structures

### ParameterDefault Enum (Runtime)

**Location**: `pyo3-stub-gen/src/type_info.rs:68-75`

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

### DefaultExpr Enum (Intermediate Representation)

**Location**: `pyo3-stub-gen-derive/src/gen_stub/parameter.rs:15-24`

```rust
/// Represents a default value expression from either Rust or Python source
#[derive(Debug, Clone)]
pub(crate) enum DefaultExpr {
    /// Rust expression that needs to be converted to Python representation at runtime
    /// Example: `vec![1, 2]`, `Number::Float`, `10`
    Rust(Expr),
    /// Python expression already in Python syntax (from Python stub)
    /// Example: `"False"`, `"[1, 2]"`, `"Number.FLOAT"`
    Python(String),
}
```

**Design rationale**:
- This enum distinguishes between the two approaches at the procedural macro level
- `Rust(Expr)`: Rust expression from `#[pyo3(signature = ...)]` that requires runtime conversion via `fmt_py_obj`
- `Python(String)`: Python expression string from `#[gen_stub_pyfunction(python = ...)]` that's already in Python syntax
- Both variants are eventually converted to `ParameterDefault::Expr(fn() -> String)` in the generated code
- This separation allows different code generation strategies for each approach

### ParameterWithKind Struct (Intermediate Representation)

**Location**: `pyo3-stub-gen-derive/src/gen_stub/parameter.rs:26-32`

```rust
/// Intermediate representation for a parameter with its kind determined
#[derive(Debug, Clone)]
pub(crate) struct ParameterWithKind {
    pub(crate) arg_info: ArgInfo,
    pub(crate) kind: ParameterKind,
    pub(crate) default_expr: Option<DefaultExpr>,
}
```

**Design rationale**:
- Used during procedural macro code generation
- `default_expr` is `Option<DefaultExpr>`, distinguishing between Rust and Python expressions
- Converted to `ParameterInfo` via the `ToTokens` trait

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

**File**: `pyo3-stub-gen-derive/src/gen_stub/parameter.rs:210-350`

The `Parameters::new_with_sig` implementation parses PyO3's `#[pyo3(signature = ...)]` attribute:

```rust
// Parse signature arguments
for sig_arg in sig.args() {
    match sig_arg {
        SignatureArg::Slash(_) => {
            // `/` delimiter - mark all previous parameters as positional-only
            for param in &mut parameters {
                param.kind = ParameterKind::PositionalOnly;
            }
        }
        SignatureArg::Star(_) => {
            // Bare `*` - parameters after this are keyword-only
            after_star = true;
        }
        SignatureArg::Assign(ident, _eq, value) => {
            // Handle parameters with default values: param = value
            let kind = if positional_only {
                ParameterKind::PositionalOnly
            } else if after_star {
                ParameterKind::KeywordOnly
            } else {
                ParameterKind::PositionalOrKeyword
            };

            let arg_info = args_map.get(&name)?.clone();

            parameters.push(ParameterWithKind {
                arg_info,
                kind,
                default_expr: Some(DefaultExpr::Rust(value.clone())),  // Rust expression
            });
        }
        SignatureArg::Ident(ident) => {
            // Handle parameters without defaults
            parameters.push(ParameterWithKind {
                arg_info,
                kind,
                default_expr: None,  // No default
            });
        }
        // ... other cases (Args, Keywords, etc.)
    }
}
```

#### 2. Code Generation

**File**: `pyo3-stub-gen-derive/src/gen_stub/parameter.rs:34-94`

The `ToTokens` implementation for `ParameterWithKind` generates code to capture default values. For Rust expressions (`DefaultExpr::Rust`), it handles two cases:

##### Case 1: RustType (Type-safe conversion)

```rust
Some(DefaultExpr::Rust(expr)) => {
    match &self.arg_info.r#type {
        TypeOrOverride::RustType { r#type } => {
            let default = if expr.to_token_stream().to_string() == "None" {
                // Special handling for None literal
                quote! { "None".to_string() }
            } else {
                // Type-checked conversion
                quote! {
                    let v: #r#type = #expr;
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
        // ...
    }
}
```

**Features**:
- Compile-time type checking: `let v: #r#type = #expr`
- Runtime conversion via `fmt_py_obj` for proper Python representation
- Special case for `None` to avoid type inference issues

##### Case 2: OverrideType (Direct string conversion)

```rust
Some(DefaultExpr::Rust(expr)) => {
    match &self.arg_info.r#type {
        TypeOrOverride::OverrideType { .. } => {
            let mut value_str = expr.to_token_stream().to_string();
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

**File**: `pyo3-stub-gen-derive/src/gen_stub/parse_python.rs:100-145`

Default values are extracted from Python AST:

```rust
let process_arg_with_default = |arg: &ast::ArgWithDefault, kind: ParameterKind| -> Result<Option<ParameterWithKind>> {
    let arg_name = arg.def.arg.to_string();

    // Convert default value from Python AST to Python string
    let default_expr = if let Some(default) = &arg.default {
        Some(DefaultExpr::Python(python_ast_to_python_string(default)?))
    } else {
        None
    };

    Ok(Some(ParameterWithKind {
        arg_info,
        kind,
        default_expr,  // Python expression as DefaultExpr::Python(String)
    }))
};
```

#### 3. Python AST to Python String Conversion

**File**: `pyo3-stub-gen-derive/src/gen_stub/parse_python.rs:288-380`

```rust
fn python_ast_to_python_string(expr: &ast::Expr) -> Result<String> {
    match expr {
        // Python literals
        ast::Expr::Constant(constant) => match &constant.value {
            ast::Constant::None => Ok("None".to_string()),
            ast::Constant::Bool(true) => Ok("True".to_string()),
            ast::Constant::Bool(false) => Ok("False".to_string()),
            ast::Constant::Int(i) => Ok(i.to_string()),
            ast::Constant::Float(f) => Ok(f.to_string()),
            ast::Constant::Str(s) => {
                // Use single quotes for Python strings, escape as needed
                if s.contains('\'') && !s.contains('"') {
                    Ok(format!("\"{}\"", s.escape_default()))
                } else {
                    Ok(format!("'{}'", s.escape_default()))
                }
            },
            ast::Constant::Ellipsis => Ok("...".to_string()),
            _ => Err(...),
        },
        // Python attribute access: MyEnum.VARIANT
        ast::Expr::Attribute(_) => expr_to_type_string(expr),
        // Python unary operations: -42
        ast::Expr::UnaryOp(unary) => {
            if matches!(unary.op, ast::UnaryOp::USub) {
                // Handle negative numbers
                if let ast::Expr::Constant(constant) = &*unary.operand {
                    match &constant.value {
                        ast::Constant::Int(i) => Ok(format!("-{}", i)),
                        ast::Constant::Float(f) => Ok(format!("-{}", f)),
                        _ => Ok("...".to_string()),
                    }
                } else {
                    Ok("...".to_string())
                }
            } else {
                Ok("...".to_string())
            }
        }
        // Lists, tuples, dicts are recursively converted
        ast::Expr::List(list) => {
            let elements: Result<Vec<_>> =
                list.elts.iter().map(python_ast_to_python_string).collect();
            Ok(format!("[{}]", elements?.join(", ")))
        }
        ast::Expr::Tuple(tuple) => {
            let elements: Result<Vec<_>> =
                tuple.elts.iter().map(python_ast_to_python_string).collect();
            let elements = elements?;
            if elements.len() == 1 {
                Ok(format!("({},)", elements[0]))
            } else {
                Ok(format!("({})", elements.join(", ")))
            }
        }
        ast::Expr::Dict(dict) => {
            let mut pairs = Vec::new();
            for (key_opt, value) in dict.keys.iter().zip(dict.values.iter()) {
                if let Some(key) = key_opt {
                    let key_str = python_ast_to_python_string(key)?;
                    let value_str = python_ast_to_python_string(value)?;
                    pairs.push(format!("{}: {}", key_str, value_str));
                } else {
                    return Ok("...".to_string());
                }
            }
            Ok(format!("{{{}}}", pairs.join(", ")))
        }
        _ => Ok("...".to_string()),
    }
}
```

**Key conversions**:
- Python `True`/`False` → String `"True"`/`"False"` (Python syntax preserved)
- Python `None` → String `"None"`
- Python `Number.FLOAT` → String `"Number.FLOAT"` (attribute access as-is)
- Python collections → Recursively converted to Python string representation
- Unsupported expressions → `"..."` (placeholder)

#### 4. Code Generation

**File**: `pyo3-stub-gen-derive/src/gen_stub/parameter.rs:82-92`

For Python-based parameters, the `DefaultExpr::Python` variant is handled:

```rust
Some(DefaultExpr::Python(py_str)) => {
    // Python expression: already in Python syntax, use directly
    quote! {
        ::pyo3_stub_gen::type_info::ParameterDefault::Expr({
            fn _fmt() -> String {
                #py_str.to_string()
            }
            _fmt
        })
    }
}
```

**Key points**:
- The Python string is used directly without any conversion
- No `fmt_py_obj` conversion needed - expression is already in Python syntax
- Boolean conversions (`True`/`False`) were already done in `python_ast_to_python_string`
- The generated closure simply returns the pre-formatted Python string

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

### 6. Python AST to Python String Conversion

**Decision**: Convert Python AST directly to Python string representation, not to Rust `syn::Expr`.

**Rationale**:
- Preserves Python syntax exactly as written in the stub
- Avoids unnecessary round-trip through Rust expression representation
- Simpler implementation - no need to map Python constructs to Rust syntax
- More predictable output - what you write in Python is what appears in the stub
- The `DefaultExpr` enum distinguishes between Rust and Python expressions at compile-time
- Both approaches converge at the `ParameterDefault::Expr(fn() -> String)` level

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
| **Default expr representation** | `DefaultExpr::Rust(syn::Expr)` | `DefaultExpr::Python(String)` |
| **Compile-time checking** | ✅ Full type checking | ⚠️ Syntax checking only |
| **Conversion timing** | Runtime via `fmt_py_obj()` | Compile-time via `python_ast_to_python_string()` |
| **Conversion result** | Python `repr()` at runtime | Python string at compile-time |
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

## Implementation Summary

The key to understanding default value handling is the `DefaultExpr` enum:

```rust
pub(crate) enum DefaultExpr {
    Rust(Expr),      // From #[pyo3(signature = ...)] - converted at runtime
    Python(String),  // From python = r#"..."# - already Python syntax
}
```

**Flow**:
1. **Compile-time** (procedural macros):
   - Rust-based: Parse signature → `DefaultExpr::Rust(expr)` → Generate code calling `fmt_py_obj(expr)`
   - Python-based: Parse Python stub → `DefaultExpr::Python(string)` → Generate code returning string directly

2. **Code generation**:
   - Both create `ParameterDefault::Expr(fn() -> String)` closures
   - Rust variant: closure calls `fmt_py_obj` at runtime
   - Python variant: closure returns pre-formatted string

3. **Runtime** (stub generation):
   - Execute closures to get Python representation strings
   - Write to `.pyi` files

## Related Documentation

- [Default Value for Class Members](./default-value-members.md) - Default values for class attributes and properties
- PyO3 documentation: [Function Signatures](https://pyo3.rs/latest/function/signature.html)
- CLAUDE.md: Python Stub Syntax Support - Details on using `gen_function_from_python!` and related macros
