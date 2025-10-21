# Procedural Macro Design Pattern

## Overview

The `pyo3-stub-gen-derive` crate implements procedural macros that extract type information from Rust code at compile-time. This document describes the consistent three-layer architecture used throughout the derive crate.

## Design Principles

### 1. Separation of Concerns

The derive crate strictly separates responsibilities into three layers:

1. **TokenStream Layer** (`src/gen_stub.rs`): Parsing and generation of proc-macro `TokenStream`
2. **Intermediate Representation Layer** (`src/gen_stub/*.rs`): Business logic and type transformations
3. **Code Generation Layer**: `ToTokens` trait implementation for generating output code

### 2. Single TokenStream Entry Point

**Rule**: Only `src/gen_stub.rs` directly manipulates `TokenStream` objects.

**Rationale**:
- Centralizes all proc-macro plumbing in one place
- Makes the codebase easier to understand and maintain
- Reduces coupling between parsing logic and business logic
- Simplifies testing of business logic (no TokenStream mocking needed)

### 3. Type-Safe Intermediate Representation

All proc-macro logic operates on strongly-typed intermediate representations (`*Info` structs), not raw `TokenStream` or `syn` types.

## Three-Layer Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│ Layer 1: Entry Point (gen_stub.rs)                             │
│                                                                 │
│  - Public proc-macro functions                                 │
│  - TokenStream parsing (TokenStream2 → syn types)              │
│  - TokenStream generation (ToTokens → TokenStream2)            │
│                                                                 │
│  Examples:                                                      │
│    pub fn pyclass(...)    -> Result<TokenStream2>             │
│    pub fn pyfunction(...)  -> Result<TokenStream2>             │
│    pub fn pymethods(...)   -> Result<TokenStream2>             │
└─────────────────────────────────────────────────────────────────┘
                           ↓ syn::parse2
┌─────────────────────────────────────────────────────────────────┐
│ Layer 2: Intermediate Representation (gen_stub/*.rs)           │
│                                                                 │
│  - *Info structs (PyClassInfo, PyFunctionInfo, etc.)           │
│  - Conversion from syn types (TryFrom implementations)         │
│  - Business logic (parameter handling, type extraction, etc.)  │
│  - Validation and error handling                               │
│                                                                 │
│  Examples:                                                      │
│    struct PyFunctionInfo { ... }                               │
│    impl TryFrom<ItemFn> for PyFunctionInfo                     │
└─────────────────────────────────────────────────────────────────┘
                           ↓ ToTokens
┌─────────────────────────────────────────────────────────────────┐
│ Layer 3: Code Generation (ToTokens implementations)            │
│                                                                 │
│  - Generate Rust code that constructs metadata                 │
│  - Use quote! macro for code generation                        │
│  - Register metadata with inventory crate                      │
│                                                                 │
│  Examples:                                                      │
│    impl ToTokens for PyFunctionInfo                            │
│    impl ToTokens for PyClassInfo                               │
└─────────────────────────────────────────────────────────────────┘
```

## Standard Flow Pattern

Every proc-macro follows this consistent pattern:

```rust
// In src/gen_stub.rs
pub fn pyfunction(attr: TokenStream2, item: TokenStream2) -> Result<TokenStream2> {
    // Step 1: Parse TokenStream to syn types
    let item_fn = parse2::<ItemFn>(item)?;

    // Step 2: Convert to intermediate representation
    let inner = PyFunctionInfo::try_from(item_fn)?;

    // Step 3: Generate output TokenStream via ToTokens
    Ok(quote! { #inner })
}
```

This pattern ensures:
- Consistent error handling
- Clear separation of parsing, transformation, and generation
- Easy to test each layer independently

## Layer 1: Entry Point (`src/gen_stub.rs`)

### Responsibilities

- Define public proc-macro entry points
- Parse attribute and item `TokenStream` into `syn` types
- Delegate to intermediate representations for processing
- Convert results back to `TokenStream` via `quote!`

### Example: `#[gen_stub_pyfunction]`

```rust
pub fn pyfunction(attr: TokenStream2, item: TokenStream2) -> Result<TokenStream2> {
    // Parse input TokenStream
    let item_fn = parse2::<ItemFn>(item)?;

    // Check for Python stub syntax in attributes
    let stub_attr = parse_gen_stub_attr(&item_fn.attrs, AttributeLocation::Function)?;

    let inner = if let Some(python_stub) = stub_attr.python {
        // Python stub syntax path
        PyFunctionInfo::from_python_stub(&python_stub, &item_fn)?
    } else {
        // Standard Rust path
        PyFunctionInfo::try_from(item_fn)?
    };

    // Generate output TokenStream
    Ok(quote! { #inner })
}
```

### Design Constraints

**DO**:
- Parse `TokenStream2` to `syn` types (e.g., `ItemFn`, `ItemStruct`, `ItemImpl`)
- Use `parse2` for parsing
- Use `quote!` for final code generation
- Return `Result<TokenStream2>` for error handling

**DON'T**:
- Implement business logic directly
- Manipulate `syn` types beyond basic parsing
- Generate code manually (use `ToTokens` instead)

## Layer 2: Intermediate Representation (`src/gen_stub/*.rs`)

### Responsibilities

- Define `*Info` structs representing extracted metadata
- Implement `TryFrom<SynType>` for conversion from `syn` types
- Contain all business logic (parameter extraction, type inference, etc.)
- Validate input and provide meaningful error messages
- Implement `ToTokens` for code generation

### Key Modules

- **`pyclass.rs`**: `PyClassInfo` - class metadata
- **`pyfunction.rs`**: `PyFunctionInfo` - function metadata
- **`pymethods.rs`**: `PyMethodsInfo` - methods metadata
- **`member.rs`**: `MemberInfo` - class member (property/attribute) metadata
- **`parameter.rs`**: `ParameterWithKind`, `Parameters` - function parameter metadata
- **`parse_python.rs`**: Python stub syntax parsing utilities

### Example: PyFunctionInfo

```rust
pub(crate) struct PyFunctionInfo {
    pub(crate) name: String,
    pub(crate) parameters: Parameters,
    pub(crate) return_type: TypeOrOverride,
    pub(crate) doc: String,
    pub(crate) original_item: ItemFn,
}

impl TryFrom<ItemFn> for PyFunctionInfo {
    type Error = syn::Error;

    fn try_from(item_fn: ItemFn) -> Result<Self> {
        // Extract function name
        let name = item_fn.sig.ident.to_string();

        // Parse parameters (including defaults from #[pyo3(signature = ...)])
        let parameters = Parameters::from_signature(&item_fn)?;

        // Extract return type
        let return_type = extract_return_type(&item_fn.sig.output, &item_fn.attrs)?;

        // Extract documentation
        let doc = extract_doc(&item_fn.attrs);

        Ok(Self {
            name,
            parameters,
            return_type,
            doc,
            original_item: item_fn,
        })
    }
}
```

### Design Constraints

**DO**:
- Create strongly-typed structs for each concept
- Implement `TryFrom` for conversion from `syn` types
- Validate all inputs and return `syn::Error` for invalid cases
- Extract and process all relevant metadata
- Keep business logic testable (independent of TokenStream)

**DON'T**:
- Directly manipulate `TokenStream` or `TokenStream2`
- Use `quote!` directly (implement `ToTokens` instead)
- Bypass validation for convenience

## Layer 3: Code Generation (`ToTokens` implementations)

### Responsibilities

- Generate Rust code that constructs metadata at runtime
- Use `quote!` macro for code generation
- Register metadata with `inventory` crate
- Preserve original item alongside metadata

### Example: PyFunctionInfo Code Generation

```rust
impl ToTokens for PyFunctionInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        let parameters = &self.parameters;  // Also implements ToTokens
        let return_type = &self.return_type;
        let doc = &self.doc;
        let original_item = &self.original_item;

        // Generate metadata registration code
        tokens.extend(quote! {
            // Preserve original function
            #original_item

            // Register metadata with inventory
            ::pyo3_stub_gen::inventory::submit! {
                ::pyo3_stub_gen::type_info::PyFunctionInfo {
                    name: #name,
                    parameters: vec![#parameters],
                    return_type: {
                        fn _type() -> ::pyo3_stub_gen::TypeInfo {
                            #return_type
                        }
                        _type
                    },
                    doc: #doc,
                }
            }
        });
    }
}
```

### Design Constraints

**DO**:
- Implement `ToTokens` for all intermediate representation types
- Use `quote!` for code generation
- Generate valid Rust code
- Preserve original items (functions, classes, etc.)
- Use closures for deferred evaluation (e.g., type info, default values)

**DON'T**:
- Perform complex logic in `ToTokens` (do it in Layer 2)
- Generate invalid Rust syntax
- Forget to preserve the original item

## Conversion Patterns

### From Rust Types to Metadata

```rust
// syn::ItemFn → PyFunctionInfo
impl TryFrom<ItemFn> for PyFunctionInfo { ... }

// syn::ItemStruct → PyClassInfo
impl TryFrom<ItemStruct> for PyClassInfo { ... }

// syn::ImplItemFn → MemberInfo (for methods)
impl MemberInfo {
    fn from_method(item: &ImplItemFn) -> Result<Self> { ... }
}
```

### From Python Stub Syntax to Metadata

```rust
// Python stub string → PyFunctionInfo
impl PyFunctionInfo {
    fn from_python_stub(stub: &str, original: &ItemFn) -> Result<Self> {
        // Parse Python stub with rustpython-parser
        let ast = parse_program(stub, "<embedded>")?;

        // Extract function definition
        let func_def = find_function_def(&ast)?;

        // Build parameters from Python AST
        let parameters = Parameters::from_python_ast(&func_def.args)?;

        // Build return type from Python AST
        let return_type = TypeOrOverride::from_python_ast(&func_def.returns)?;

        Ok(Self { ... })
    }
}
```

## Error Handling

All layers use `syn::Error` for compile-time errors:

```rust
// Layer 1: Entry point
pub fn pyfunction(attr: TokenStream2, item: TokenStream2) -> Result<TokenStream2> {
    let item_fn = parse2::<ItemFn>(item)
        .map_err(|e| syn::Error::new(e.span(), "Expected function definition"))?;
    // ...
}

// Layer 2: Intermediate representation
impl TryFrom<ItemFn> for PyFunctionInfo {
    fn try_from(item: ItemFn) -> Result<Self> {
        if item.sig.asyncness.is_some() {
            return Err(syn::Error::new_spanned(
                &item.sig.asyncness,
                "Async functions are not yet supported"
            ));
        }
        // ...
    }
}
```

This provides:
- Clear error messages at compile-time
- Proper span information for error highlighting
- Consistent error handling across all layers

## Adding New Functionality

When adding new features, follow these steps:

### Step 1: Define Intermediate Representation

Create or extend `*Info` structs in `gen_stub/*.rs`:

```rust
// In src/gen_stub/myfeature.rs
pub(crate) struct MyFeatureInfo {
    pub(crate) field1: String,
    pub(crate) field2: TypeInfo,
    // ...
}
```

### Step 2: Implement Conversion from syn

```rust
impl TryFrom<ItemSomething> for MyFeatureInfo {
    type Error = syn::Error;

    fn try_from(item: ItemSomething) -> Result<Self> {
        // Extract and validate data
        // ...
        Ok(Self { ... })
    }
}
```

### Step 3: Implement Code Generation

```rust
impl ToTokens for MyFeatureInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(quote! {
            // Generate metadata registration code
            // ...
        });
    }
}
```

### Step 4: Add Entry Point

```rust
// In src/gen_stub.rs
pub fn my_feature(attr: TokenStream2, item: TokenStream2) -> Result<TokenStream2> {
    let item = parse2::<ItemSomething>(item)?;
    let info = MyFeatureInfo::try_from(item)?;
    Ok(quote! { #info })
}
```

### Step 5: Export Macro

```rust
// In src/lib.rs
#[proc_macro_attribute]
pub fn gen_stub_my_feature(attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_stub::my_feature(attr.into(), item.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
```

## Testing Strategy

### Unit Tests

Test intermediate representations independently:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse2;

    #[test]
    fn test_function_parsing() {
        let input = quote! {
            fn my_func(x: i32) -> String {
                x.to_string()
            }
        };

        let item_fn = parse2::<ItemFn>(input).unwrap();
        let info = PyFunctionInfo::try_from(item_fn).unwrap();

        assert_eq!(info.name, "my_func");
        assert_eq!(info.parameters.len(), 1);
    }
}
```

### Integration Tests

Test complete macro expansion using `insta` for snapshot testing:

```rust
#[test]
fn test_pyfunction_macro() {
    let input = quote! {
        #[gen_stub_pyfunction]
        #[pyfunction]
        fn example(x: i32) -> String {
            x.to_string()
        }
    };

    let output = pyfunction(quote!(), input).unwrap();
    insta::assert_snapshot!(output.to_string());
}
```

## Common Patterns

### Optional Attributes

```rust
// Extract optional attribute
let default = parse_gen_stub_default(&item.attrs)?;

// Use in struct
pub(crate) struct Info {
    pub(crate) default: Option<Expr>,
}

// Generate code conditionally
let default_tokens = default.map(|d| quote! { Some(#d) })
    .unwrap_or(quote! { None });
```

### Type Extraction

```rust
// Extract type from field
let ty = &field.ty;

// Convert to TypeOrOverride
let type_or_override = TypeOrOverride::from_syn_type(ty)?;

// Generate type info code
quote! {
    fn _type() -> ::pyo3_stub_gen::TypeInfo {
        #type_or_override
    }
}
```

### Closure-based Deferred Evaluation

For values that need runtime evaluation (e.g., default values):

```rust
// Generate closure that evaluates at runtime
let default_expr = if let Some(expr) = default {
    quote! {
        ::pyo3_stub_gen::type_info::ParameterDefault::Expr({
            fn _eval() -> String {
                ::pyo3_stub_gen::util::fmt_py_obj(#expr)
            }
            _eval
        })
    }
} else {
    quote! { ::pyo3_stub_gen::type_info::ParameterDefault::None }
};
```

## Anti-Patterns

### ❌ Direct TokenStream Manipulation Outside gen_stub.rs

```rust
// BAD: Don't do this in gen_stub/myfeature.rs
pub fn process(tokens: TokenStream2) -> TokenStream2 {
    // Violates layer separation
}
```

### ❌ Using quote! in Intermediate Representation

```rust
// BAD: Don't generate code in business logic
impl MyInfo {
    fn process(&self) -> TokenStream2 {
        quote! { ... }  // Wrong layer!
    }
}

// GOOD: Implement ToTokens instead
impl ToTokens for MyInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(quote! { ... });
    }
}
```

### ❌ Skipping Intermediate Representation

```rust
// BAD: Don't go directly from parsing to code generation
pub fn myfeature(attr: TokenStream2, item: TokenStream2) -> Result<TokenStream2> {
    let item = parse2::<ItemFn>(item)?;
    // Direct generation - hard to test and maintain
    Ok(quote! { ... })
}

// GOOD: Use intermediate representation
pub fn myfeature(attr: TokenStream2, item: TokenStream2) -> Result<TokenStream2> {
    let item = parse2::<ItemFn>(item)?;
    let info = MyFeatureInfo::try_from(item)?;
    Ok(quote! { #info })
}
```

## Summary

The three-layer architecture provides:

1. **Clear separation of concerns**: Each layer has a well-defined responsibility
2. **Testability**: Business logic can be tested without TokenStream mocking
3. **Maintainability**: Changes are localized to appropriate layers
4. **Consistency**: All macros follow the same pattern
5. **Error handling**: Compile-time errors with proper span information

When adding new functionality, always follow this pattern:
- **Layer 1** (gen_stub.rs): Parse TokenStream, delegate, generate output
- **Layer 2** (gen_stub/*.rs): Define `*Info` structs, implement `TryFrom`, contain business logic
- **Layer 3** (ToTokens): Generate code that registers metadata

## Related Documentation

- [Architecture](./architecture.md) - Overall system architecture
- [Default Value for Function Arguments](./default-value-arguments.md) - Example of the pattern in practice
- [Default Value for Class Members](./default-value-members.md) - Another example implementation
