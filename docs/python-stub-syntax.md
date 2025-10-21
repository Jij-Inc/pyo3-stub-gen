# Python Stub Syntax Support

## Overview

pyo3-stub-gen supports writing type information directly in Python stub syntax (`.pyi` format) instead of relying solely on automatic Rust-to-Python type translation. This feature is essential for:

- Complex Python types that don't map cleanly from Rust (e.g., `collections.abc.Callable`, `typing.Protocol`)
- Function overloads (`@overload` decorator support)
- Type overrides when automatic translation is insufficient
- Providing more Pythonic type annotations

## Motivation

While automatic type translation works for most cases, some Python type patterns cannot be represented in Rust:

```python
# Python stub with Callable - no direct Rust equivalent
def process(callback: collections.abc.Callable[[str], int]) -> int: ...

# Function overloads - same name, different signatures
@overload
def get_value(key: str) -> str: ...
@overload
def get_value(key: int) -> int: ...
```

Python stub syntax support allows developers to specify these types directly in familiar Python notation.

## Three Approaches

### 1. Inline Python Parameter (Recommended for Single Functions)

Use the `python` parameter in `#[gen_stub_pyfunction]` to specify the complete function signature in Python stub syntax:

```rust
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyfunction(python = r#"
    import collections.abc
    import typing

    def fn_with_callback(
        callback: collections.abc.Callable[[str], typing.Any]
    ) -> collections.abc.Callable[[str], typing.Any]:
        """Example using python parameter."""
"#)]
#[pyfunction]
pub fn fn_with_callback<'a>(callback: Bound<'a, PyAny>) -> PyResult<Bound<'a, PyAny>> {
    callback.call1(("Hello!",))?;
    Ok(callback)
}
```

**Generated stub:**
```python
import collections.abc
import typing

def fn_with_callback(
    callback: collections.abc.Callable[[str], typing.Any]
) -> collections.abc.Callable[[str], typing.Any]:
    """Example using python parameter."""
```

**Features:**
- ✅ Complete control over the generated signature
- ✅ Supports complex types like `Callable`, `Protocol`, generics
- ✅ Allows custom docstrings in the stub
- ✅ Import statements are automatically extracted and placed at module level

**When to use:**
- Need to specify complex Python types
- Want complete control over the stub signature
- Single function or method needs override

### 2. Function Stub Generation Macro (For Overloads)

Use `gen_function_from_python!` inside `submit!` block to add additional type signatures for the same function:

```rust
use pyo3::prelude::*;
use pyo3_stub_gen::{derive::*, inventory::submit};

// Rust implementation (also generates one stub signature automatically)
#[gen_stub_pyfunction]
#[pyfunction]
pub fn overload_example(x: f64) -> f64 {
    x + 1.0
}

// Additional overload signature
submit! {
    gen_function_from_python! {
        r#"
        def overload_example(x: int) -> int:
            """Overload for integer input"""
        "#
    }
}
```

**Generated stub:**
```python
from typing import overload

@overload
def overload_example(x: float) -> float: ...

@overload
def overload_example(x: int) -> int:
    """Overload for integer input"""
```

**Features:**
- ✅ Ideal for function overloads (`@overload` decorator)
- ✅ Keeps type definitions separate from implementation
- ✅ Allows multiple signatures for the same function name

**When to use:**
- Need function overloads (multiple type signatures)
- Want to supplement auto-generated signatures
- Separate type definitions from implementation

### 3. Methods Stub Generation Macro (For Class Methods)

Use `gen_methods_from_python!` to define additional method signatures for a class:

```rust
use pyo3::prelude::*;
use pyo3_stub_gen::{derive::*, inventory::submit};

#[gen_stub_pyclass]
#[pyclass]
pub struct Calculator {}

#[gen_stub_pymethods]
#[pymethods]
impl Calculator {
    fn add(&self, x: f64) -> f64 {
        x + 1.0
    }
}

// Additional overload for integer type
submit! {
    gen_methods_from_python! {
        r#"
        class Calculator:
            def add(self, x: int) -> int:
                """Add operation for integers"""
        "#
    }
}
```

**Generated stub:**
```python
from typing import overload

class Calculator:
    @overload
    def add(self, x: float) -> float: ...

    @overload
    def add(self, x: int) -> int:
        """Add operation for integers"""
```

**Features:**
- ✅ Supports method overloads within classes
- ✅ Can define multiple methods in a single macro call
- ✅ Integrates with auto-generated class definitions

**When to use:**
- Need method overloads in classes
- Want to add alternative signatures for existing methods
- Supplement auto-generated class stubs

## How `submit!` and Overloads Work

### Automatic `submit!` Generation

The `#[gen_stub_pyfunction]`, `#[gen_stub_pyclass]`, and `#[gen_stub_pymethods]` macros automatically generate `submit!` blocks internally to register type information with the `inventory` crate:

```rust
// You write:
#[gen_stub_pyfunction]
#[pyfunction]
pub fn my_func(x: i32) -> i32 { x + 1 }

// Macro generates (simplified):
#[pyfunction]
pub fn my_func(x: i32) -> i32 { x + 1 }

inventory::submit! {
    PyFunctionInfo {
        name: "my_func",
        parameters: /* ... */,
        return_type: /* ... */,
        /* ... */
    }
}
```

### Manual `submit!` for Overloads

You can add additional `submit!` blocks to register alternative type signatures:

```rust
// First signature (auto-generated from #[gen_stub_pyfunction])
#[gen_stub_pyfunction]
#[pyfunction]
pub fn process(x: f64) -> f64 { x }

// Second signature (manual submit!)
submit! {
    gen_function_from_python! {
        r#"
        def process(x: int) -> int: ...
        "#
    }
}
```

**Result**: Two `PyFunctionInfo` entries are registered for "process", causing the stub generator to interpret them as overloads and generate `@overload` decorators.

### Overload Detection

The stub generator detects overloads by name:

```rust
// During stub generation:
if function_signatures.len() > 1 {
    // Multiple signatures for same name → generate @overload
    for signature in &function_signatures[..function_signatures.len() - 1] {
        write!("@overload\n{}", signature);
    }
    // Last signature without @overload (implementation signature)
    write!("{}", function_signatures.last());
}
```

## Python Stub Parsing

### rustpython-parser Integration

Python stub syntax is parsed at compile-time using the `rustpython-parser` crate:

```rust
use rustpython_parser::{parse_program, ast};

let python_code = r#"
    def my_function(x: int, y: str = 'default') -> bool:
        """Function docstring."""
"#;

// Parse Python code to AST
let module = parse_program(python_code, "<embedded>")
    .map_err(|e| syn::Error::new(span, format!("Failed to parse Python: {}", e)))?;

// Extract function definition
for stmt in &module.statements {
    if let ast::Stmt::FunctionDef(func_def) = stmt {
        // Extract name, parameters, return type, docstring
        process_function(func_def)?;
    }
}
```

### AST to Metadata Conversion

The parser extracts key information from Python AST:

**Function Name:**
```rust
let name = func_def.name.to_string();
```

**Parameters:**
```rust
for arg in &func_def.args.posonlyargs {
    // Positional-only parameters
}
for arg in &func_def.args.args {
    // Regular parameters
}
for arg in &func_def.args.kwonlyargs {
    // Keyword-only parameters
}
```

**Parameter Types:**
```rust
fn extract_type_annotation(annotation: &ast::Expr) -> TypeInfo {
    match annotation {
        ast::Expr::Name(name) => {
            // Simple type: int, str, bool
            TypeInfo::builtin(&name.id)
        }
        ast::Expr::Subscript(subscript) => {
            // Generic type: list[int], dict[str, int]
            TypeInfo::with_generic(/* ... */)
        }
        ast::Expr::BinOp(binop) if binop.op == BitOr => {
            // Union type: int | str
            TypeInfo::union(/* ... */)
        }
        // ... other patterns
    }
}
```

**Default Values:**
```rust
if let Some(default_expr) = &arg.default {
    let default_str = python_ast_to_python_string(default_expr)?;
    // Store as DefaultExpr::Python(string)
}
```

See [Default Value for Function Arguments](./default-value-arguments.md#python-based-approach) for detailed default value handling.

**Return Type:**
```rust
if let Some(returns) = &func_def.returns {
    extract_type_annotation(returns)
} else {
    TypeInfo::none()  // No annotation → None
}
```

**Docstring:**
```rust
fn extract_docstring(body: &[ast::Stmt]) -> String {
    if let Some(ast::Stmt::Expr(expr)) = body.first() {
        if let ast::Expr::Constant(constant) = &expr.value {
            if let ast::Constant::Str(s) = &constant.value {
                return s.clone();
            }
        }
    }
    String::new()
}
```

## Import Handling

### Automatic Import Extraction

Import statements in Python stub syntax are automatically extracted and placed at module level:

```rust
#[gen_stub_pyfunction(python = r#"
    import collections.abc
    import typing
    from datetime import datetime

    def my_func(x: collections.abc.Callable[[int], str]) -> datetime:
        """Example with imports."""
"#)]
```

**Generated stub:**
```python
import collections.abc
import typing
from datetime import datetime

def my_func(x: collections.abc.Callable[[int], str]) -> datetime:
    """Example with imports."""
```

### Import Deduplication

The stub generator deduplicates imports across all functions/classes:

```rust
// Multiple functions use typing.Callable
// → Only one "import typing" in generated stub
```

### Standard Library Imports

Common imports are added automatically when needed:

- `import typing` - When using `typing.Any`, `typing.Sequence`, etc.
- `import collections.abc` - When using `collections.abc.Callable`, etc.
- `import numpy` - When using NumPy types (with `numpy` feature)

## RustType Marker

### Overview

The `pyo3_stub_gen.RustType["TypeName"]` marker allows referencing Rust types within Python stub syntax:

```rust
use pyo3::prelude::*;
use pyo3_stub_gen::{derive::*, inventory::submit};

submit! {
    gen_function_from_python! {
        r#"
        def sum_list(values: pyo3_stub_gen.RustType["Vec<i32>"]) -> pyo3_stub_gen.RustType["i32"]:
            """Sum a list of integers"""
        "#
    }
}
```

### How It Works

During Python stub parsing, `RustType["TypeName"]` is detected and replaced with the Rust type's `PyStubType` implementation:

```rust
fn extract_type_annotation(expr: &ast::Expr) -> TypeInfo {
    match expr {
        ast::Expr::Subscript(subscript) => {
            if is_rust_type_marker(subscript) {
                // Extract "Vec<i32>" from RustType["Vec<i32>"]
                let rust_type_name = extract_rust_type_name(subscript)?;

                // Parse as Rust type
                let rust_type: syn::Type = syn::parse_str(&rust_type_name)?;

                // Use PyStubType trait implementation
                return TypeInfo::from_rust_type(&rust_type);
            }
            // ... other subscript handling
        }
        // ...
    }
}
```

### Type Expansion

The marker is expanded according to the context:

**For function arguments (input types):**
```rust
RustType["Vec<i32>"]
→ Vec::<i32>::type_input()
→ TypeInfo::with_generic("typing.Sequence", vec![TypeInfo::builtin("int")])
→ typing.Sequence[int]
```

**For return types (output types):**
```rust
RustType["Vec<i32>"]
→ Vec::<i32>::type_output()
→ TypeInfo::with_generic("list", vec![TypeInfo::builtin("int")])
→ list[int]
```

### Use Cases

**Generic Types:**
```rust
submit! {
    gen_function_from_python! {
        r#"
        def process(
            data: pyo3_stub_gen.RustType["HashMap<String, Vec<i32>>"]
        ) -> pyo3_stub_gen.RustType["Vec<String>"]:
            """Process data."""
        "#
    }
}
```

**Custom Types:**
```rust
#[gen_stub_pyclass]
#[pyclass]
struct MyClass;

submit! {
    gen_function_from_python! {
        r#"
        def create() -> pyo3_stub_gen.RustType["MyClass"]:
            """Create MyClass instance."""
        "#
    }
}
```

**Ensuring Type Consistency:**
```rust
// Ensures Python stub matches Rust type exactly
submit! {
    gen_function_from_python! {
        r#"
        def get_config() -> pyo3_stub_gen.RustType["Arc<Config>"]:
            """Get shared config."""
        "#
    }
}
```

## Advanced Patterns

### Mixing Automatic and Manual Signatures

```rust
#[gen_stub_pyclass]
#[pyclass]
pub struct DataProcessor;

#[gen_stub_pymethods]
#[pymethods]
impl DataProcessor {
    // Auto-generated signature
    fn process_float(&self, x: f64) -> f64 {
        x * 2.0
    }

    // Manual signature with complex types
    #[gen_stub(python = r#"
        import collections.abc

        def process_callback(
            self,
            callback: collections.abc.Callable[[float], float]
        ) -> float:
            """Process with callback"""
    "#)]
    fn process_callback(&self, callback: Bound<'_, PyAny>) -> f64 {
        // Implementation
        0.0
    }
}
```

### Overload with RustType

```rust
#[gen_stub_pyfunction]
#[pyfunction]
pub fn convert(x: f64) -> f64 {
    x
}

submit! {
    gen_function_from_python! {
        r#"
        def convert(x: pyo3_stub_gen.RustType["Vec<f64>"]) -> pyo3_stub_gen.RustType["Vec<f64>"]:
            """Convert list of floats"""
        "#
    }
}
```

**Generated stub:**
```python
from typing import overload

@overload
def convert(x: float) -> float: ...

@overload
def convert(x: typing.Sequence[float]) -> list[float]:
    """Convert list of floats"""
```

### Protocol Types

```rust
submit! {
    gen_function_from_python! {
        r#"
        import typing

        class Comparable(typing.Protocol):
            def __lt__(self, other: typing.Any) -> bool: ...

        def sort_items(items: list[Comparable]) -> list[Comparable]:
            """Sort comparable items"""
        "#
    }
}
```

## Implementation Details

### Parser Module Structure

```
pyo3-stub-gen-derive/src/gen_stub/parse_python/
├── mod.rs              # Main parsing utilities
├── pyfunction.rs       # Function definition parsing
└── pyclass.rs          # Class definition parsing (future)
```

### Key Functions

**`parse_python_function_stub`** (`parse_python/pyfunction.rs`):
```rust
pub fn parse_python_function_stub(
    stub_str: &str,
    original_item: &ItemFn,
) -> Result<PyFunctionInfo> {
    // 1. Parse Python code to AST
    let module = parse_program(stub_str, "<embedded>")?;

    // 2. Find function definition
    let func_def = find_function_def(&module)?;

    // 3. Extract name
    let name = func_def.name.to_string();

    // 4. Build parameters
    let parameters = build_parameters_from_ast(&func_def.args)?;

    // 5. Extract return type
    let return_type = extract_return_type_from_ast(&func_def.returns)?;

    // 6. Extract docstring
    let doc = extract_docstring(&func_def.body);

    // 7. Extract imports
    let imports = extract_imports(&module)?;

    Ok(PyFunctionInfo {
        name,
        parameters,
        return_type,
        doc,
        imports,
        original_item: original_item.clone(),
    })
}
```

**`build_parameters_from_ast`** (`parse_python/mod.rs`):
```rust
fn build_parameters_from_ast(args: &ast::Arguments) -> Result<Parameters> {
    let mut parameters = Vec::new();

    // Positional-only parameters (before /)
    for arg in &args.posonlyargs {
        parameters.push(ParameterWithKind {
            arg_info: extract_arg_info(arg)?,
            kind: ParameterKind::PositionalOnly,
            default_expr: extract_default(arg)?,
        });
    }

    // Regular parameters
    for arg in &args.args {
        parameters.push(ParameterWithKind {
            arg_info: extract_arg_info(arg)?,
            kind: ParameterKind::PositionalOrKeyword,
            default_expr: extract_default(arg)?,
        });
    }

    // *args
    if let Some(vararg) = &args.vararg {
        parameters.push(ParameterWithKind {
            arg_info: extract_arg_info(vararg)?,
            kind: ParameterKind::VarPositional,
            default_expr: None,
        });
    }

    // Keyword-only parameters (after *)
    for arg in &args.kwonlyargs {
        parameters.push(ParameterWithKind {
            arg_info: extract_arg_info(arg)?,
            kind: ParameterKind::KeywordOnly,
            default_expr: extract_default(arg)?,
        });
    }

    // **kwargs
    if let Some(kwarg) = &args.kwarg {
        parameters.push(ParameterWithKind {
            arg_info: extract_arg_info(kwarg)?,
            kind: ParameterKind::VarKeyword,
            default_expr: None,
        });
    }

    Ok(Parameters::new_from_vec(parameters))
}
```

## Error Handling

### Parse Errors

Python syntax errors are reported at compile-time with helpful messages:

```rust
#[gen_stub_pyfunction(python = r#"
    def my_func(x: int  # Missing closing paren
"#)]
```

**Error:**
```
error: Failed to parse Python stub: Expected ')', found newline at line 1
  --> src/lib.rs:10:5
   |
10 | #[gen_stub_pyfunction(python = r#"
   |   ^^^^^^^^^^^^^^^^^^^
```

### Type Annotation Errors

Missing or invalid type annotations:

```rust
#[gen_stub_pyfunction(python = r#"
    def my_func(x):  # Missing type annotation
        """Example"""
"#)]
```

**Error:**
```
error: Parameter 'x' missing type annotation
  --> src/lib.rs:10:5
```

### RustType Parse Errors

Invalid Rust type in `RustType` marker:

```rust
submit! {
    gen_function_from_python! {
        r#"
        def my_func(x: pyo3_stub_gen.RustType["Vec<>"]): ...  # Invalid Rust syntax
        "#
    }
}
```

**Error:**
```
error: Failed to parse Rust type "Vec<>": expected type, found `>`
```

## Testing

### Unit Tests

Test Python stub parsing:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let stub = r#"
            def my_func(x: int) -> str:
                """Example function"""
        "#;

        let result = parse_python_function_stub(stub, &dummy_item_fn());
        assert!(result.is_ok());

        let info = result.unwrap();
        assert_eq!(info.name, "my_func");
        assert_eq!(info.parameters.len(), 1);
        assert_eq!(info.doc, "Example function");
    }

    #[test]
    fn test_parse_with_imports() {
        let stub = r#"
            import typing
            import collections.abc

            def my_func(x: collections.abc.Callable[[int], str]) -> typing.Any:
                """With imports"""
        "#;

        let result = parse_python_function_stub(stub, &dummy_item_fn());
        assert!(result.is_ok());

        let info = result.unwrap();
        assert!(info.imports.contains(&"typing"));
        assert!(info.imports.contains(&"collections.abc"));
    }
}
```

### Snapshot Tests

Use `insta` for snapshot testing:

```rust
#[test]
fn test_gen_function_from_python_output() {
    let input = quote! {
        gen_function_from_python! {
            r#"
            def example(x: int, y: str = 'default') -> bool:
                """Example function"""
            "#
        }
    };

    let output = gen_function_from_python_impl(input).unwrap();
    insta::assert_snapshot!(output.to_string());
}
```

## Best Practices

### 1. Use Python Stub Syntax for Complex Types

```rust
// Good: Use Python syntax for Callable
#[gen_stub_pyfunction(python = r#"
    import collections.abc
    def my_func(cb: collections.abc.Callable[[int], str]) -> None: ...
"#)]

// Avoid: Complex override attributes
#[gen_stub_pyfunction]
#[gen_stub(override_return_type(type_repr="...", imports=(...)))]
```

### 2. Prefer Automatic Generation When Possible

```rust
// Good: Let automatic translation handle simple types
#[gen_stub_pyfunction]
#[pyfunction]
fn simple_func(x: i32) -> String { /* ... */ }

// Avoid: Manual specification for simple types
#[gen_stub_pyfunction(python = r#"
    def simple_func(x: int) -> str: ...
"#)]
```

### 3. Use RustType for Consistency

```rust
// Good: Use RustType to ensure consistency
submit! {
    gen_function_from_python! {
        r#"
        def process(data: pyo3_stub_gen.RustType["MyConfig"]) -> None: ...
        "#
    }
}

// Avoid: Hardcoding type that might change
submit! {
    gen_function_from_python! {
        r#"
        def process(data: MyConfig) -> None: ...
        "#
    }
}
```

### 4. Document Overloads

```rust
// Good: Document why overload is needed
// Overload for integer input (more efficient path)
submit! {
    gen_function_from_python! {
        r#"
        def process(x: int) -> int:
            """Process integer (optimized)"""
        "#
    }
}
```

## Limitations

### 1. No Class-Level Python Stubs (Yet)

Currently, `gen_stub_pyclass(python = "...")` is not supported. Use `gen_methods_from_python!` instead for class methods.

### 2. No Type Validation

Python type annotations in stub syntax are not validated against Rust implementation at compile-time. Type checking happens only when users run mypy/pyright on the generated stubs.

### 3. Limited AST Pattern Support

Some complex Python type patterns may not be fully supported. Fallback to `typing.Any` or manual override if needed.

## Related Documentation

- [Architecture](./architecture.md) - Overall system architecture
- [Type System](./type-system.md) - Rust to Python type mappings
- [Default Value for Function Arguments](./default-value-arguments.md) - Parameter default values
- [Procedural Macro Design](./procedural-macro-design.md) - How proc-macros work internally
