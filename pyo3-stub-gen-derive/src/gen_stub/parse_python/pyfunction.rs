//! Parse Python function stub syntax and generate PyFunctionInfo

use rustpython_parser::{ast, Parse};
use syn::{parse::Parse as SynParse, parse::ParseStream, Error, LitStr, Result};

use super::{
    build_parameters_from_ast, dedent, extract_deprecated_from_decorators, extract_docstring,
    extract_return_type, has_overload_decorator,
};
use crate::gen_stub::pyfunction::PyFunctionInfo;

/// Input for gen_function_from_python! macro
pub struct GenFunctionFromPythonInput {
    module: Option<String>,
    python_stub: LitStr,
}

impl SynParse for GenFunctionFromPythonInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // Check if first token is an identifier (for module parameter)
        if input.peek(syn::Ident) {
            let key: syn::Ident = input.parse()?;
            if key == "module" {
                let _: syn::token::Eq = input.parse()?;
                let value: LitStr = input.parse()?;
                let _: syn::token::Comma = input.parse()?;
                let python_stub: LitStr = input.parse()?;
                return Ok(Self {
                    module: Some(value.value()),
                    python_stub,
                });
            } else {
                return Err(Error::new(
                    key.span(),
                    format!(
                        "Unknown parameter: {}. Expected 'module' or a string literal",
                        key
                    ),
                ));
            }
        }

        // No module parameter, just parse the string literal
        let python_stub: LitStr = input.parse()?;
        Ok(Self {
            module: None,
            python_stub,
        })
    }
}

/// Intermediate representation for Python function stub
pub struct PythonFunctionStub {
    pub func_def: ast::StmtFunctionDef,
    pub imports: Vec<String>,
    pub is_async: bool,
    pub is_overload: bool,
}

impl TryFrom<PythonFunctionStub> for PyFunctionInfo {
    type Error = syn::Error;

    fn try_from(stub: PythonFunctionStub) -> Result<Self> {
        let func_name = stub.func_def.name.to_string();

        // Extract docstring
        let doc = extract_docstring(&stub.func_def);

        // Build Parameters directly from Python AST with proper kind classification
        let parameters = build_parameters_from_ast(&stub.func_def.args, &stub.imports)?;

        // Extract return type
        let return_type = extract_return_type(&stub.func_def.returns, &stub.imports)?;

        // Try to extract deprecated decorator
        let deprecated = extract_deprecated_from_decorators(&stub.func_def.decorator_list);

        // Note: type_ignored (# type: ignore comments) cannot be extracted from Python AST
        // as rustpython-parser doesn't preserve comments

        // Construct PyFunctionInfo
        Ok(PyFunctionInfo {
            name: func_name,
            parameters, // Use pre-built Parameters from Python AST
            r#return: return_type,
            doc,
            module: None,
            is_async: stub.is_async,
            deprecated,
            type_ignored: None,
            is_overload: stub.is_overload,
            index: 0, // Will be set by caller when generating multiple overloads
        })
    }
}

/// Parse Python stub string and return PyFunctionInfo
pub fn parse_python_function_stub(input: LitStr) -> Result<PyFunctionInfo> {
    let stub_content = input.value();

    // Remove common indentation to allow indented Python code in raw strings
    let dedented_content = dedent(&stub_content);

    // Parse Python code using rustpython-parser
    let parsed = ast::Suite::parse(&dedented_content, "<stub>")
        .map_err(|e| Error::new(input.span(), format!("Failed to parse Python stub: {}", e)))?;

    // Extract imports and function definitions
    let mut imports = Vec::new();
    let mut function: Option<(ast::StmtFunctionDef, bool)> = None;

    for stmt in parsed {
        match stmt {
            ast::Stmt::Import(import_stmt) => {
                for alias in &import_stmt.names {
                    imports.push(alias.name.to_string());
                }
            }
            ast::Stmt::ImportFrom(import_from_stmt) => {
                if let Some(module) = &import_from_stmt.module {
                    imports.push(module.to_string());
                }
            }
            ast::Stmt::FunctionDef(func_def) => {
                if function.is_some() {
                    return Err(Error::new(
                        input.span(),
                        "Multiple function definitions found. Only one function is allowed per gen_function_from_python! call",
                    ));
                }
                function = Some((func_def, false));
            }
            ast::Stmt::AsyncFunctionDef(func_def) => {
                if function.is_some() {
                    return Err(Error::new(
                        input.span(),
                        "Multiple function definitions found. Only one function is allowed per gen_function_from_python! call",
                    ));
                }
                // Convert AsyncFunctionDef to FunctionDef for uniform processing
                let sync_func = ast::StmtFunctionDef {
                    range: func_def.range,
                    name: func_def.name,
                    type_params: func_def.type_params,
                    args: func_def.args,
                    body: func_def.body,
                    decorator_list: func_def.decorator_list,
                    returns: func_def.returns,
                    type_comment: func_def.type_comment,
                };
                function = Some((sync_func, true));
            }
            _ => {
                // Ignore other statements
            }
        }
    }

    // Check that exactly one function is defined
    let (func_def, is_async) = function
        .ok_or_else(|| Error::new(input.span(), "No function definition found in Python stub"))?;

    // Check if function has @overload decorator
    let is_overload = has_overload_decorator(&func_def.decorator_list);

    // Generate PyFunctionInfo using TryFrom
    let stub = PythonFunctionStub {
        func_def,
        imports,
        is_async,
        is_overload,
    };
    PyFunctionInfo::try_from(stub)
}

/// Parse multiple overload function definitions from Python stub string
/// Used for the `python_overload` parameter
pub fn parse_python_overload_stubs(
    input: LitStr,
    expected_function_name: &str,
) -> Result<Vec<PyFunctionInfo>> {
    let stub_content = input.value();
    let dedented_content = dedent(&stub_content);

    // Parse Python code using rustpython-parser
    let parsed = ast::Suite::parse(&dedented_content, "<stub>")
        .map_err(|e| Error::new(input.span(), format!("Failed to parse Python stub: {}", e)))?;

    // Extract imports and function definitions
    let mut imports = Vec::new();
    let mut functions: Vec<(ast::StmtFunctionDef, bool)> = Vec::new();

    for stmt in parsed {
        match stmt {
            ast::Stmt::Import(import_stmt) => {
                for alias in &import_stmt.names {
                    imports.push(alias.name.to_string());
                }
            }
            ast::Stmt::ImportFrom(import_from_stmt) => {
                if let Some(module) = &import_from_stmt.module {
                    imports.push(module.to_string());
                }
            }
            ast::Stmt::FunctionDef(func_def) => {
                functions.push((func_def, false));
            }
            ast::Stmt::AsyncFunctionDef(func_def) => {
                // Convert AsyncFunctionDef to FunctionDef for uniform processing
                let sync_func = ast::StmtFunctionDef {
                    range: func_def.range,
                    name: func_def.name,
                    type_params: func_def.type_params,
                    args: func_def.args,
                    body: func_def.body,
                    decorator_list: func_def.decorator_list,
                    returns: func_def.returns,
                    type_comment: func_def.type_comment,
                };
                functions.push((sync_func, true));
            }
            _ => {
                // Ignore other statements
            }
        }
    }

    // Check that at least one function is defined
    if functions.is_empty() {
        return Err(Error::new(
            input.span(),
            "No function definition found in python_overload parameter",
        ));
    }

    // Validate all functions
    let mut result = Vec::new();
    for (func_def, is_async) in functions {
        let func_name = func_def.name.to_string();

        // Validate: function name must match expected name
        if func_name != expected_function_name {
            return Err(Error::new(
                input.span(),
                format!(
                    "Function name '{}' in python_overload does not match Rust function name '{}'. Please ensure all overload function names match the Rust function name.",
                    func_name, expected_function_name
                ),
            ));
        }

        // Validate: all functions must have @overload decorator
        let is_overload = has_overload_decorator(&func_def.decorator_list);
        if !is_overload {
            return Err(Error::new(
                input.span(),
                format!(
                    "Function '{}' in python_overload must have @overload decorator",
                    func_name
                ),
            ));
        }

        // Create PyFunctionInfo
        let stub = PythonFunctionStub {
            func_def,
            imports: imports.clone(),
            is_async,
            is_overload,
        };
        result.push(PyFunctionInfo::try_from(stub)?);
    }

    Ok(result)
}

/// Parse gen_function_from_python! input with optional module parameter
pub fn parse_gen_function_from_python_input(
    input: GenFunctionFromPythonInput,
) -> Result<PyFunctionInfo> {
    let mut info = parse_python_function_stub(input.python_stub)?;

    // Set module if provided
    if let Some(module) = input.module {
        info.module = Some(module);
    }

    Ok(info)
}

#[cfg(test)]
mod test {
    use super::*;
    use proc_macro2::TokenStream as TokenStream2;
    use quote::{quote, ToTokens};

    #[test]
    fn test_basic_function() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            def foo(x: int) -> int:
                """A simple function"""
            "#
        })?;
        let info = parse_python_function_stub(stub_str)?;
        let out = info.to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r#"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "foo",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "x",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "int".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from([]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "int".to_string(),
                source_module: None,
                import: ::std::collections::HashSet::from([]),
                type_refs: ::std::collections::HashMap::new(),
            },
            doc: "A simple function",
            module: None,
            is_async: false,
            deprecated: None,
            type_ignored: None,
            is_overload: false,
            file: file!(),
            line: line!(),
            column: column!(),
            index: 0usize,
        }
        "#);
        Ok(())
    }

    #[test]
    fn test_function_with_imports() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            import typing
            from collections.abc import Callable

            def process(func: Callable[[str], int]) -> typing.Optional[int]:
                """Process a callback function"""
            "#
        })?;
        let info = parse_python_function_stub(stub_str)?;
        let out = info.to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r#"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "process",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "func",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "Callable[[str], int]".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from([
                            "typing".into(),
                            "collections.abc".into(),
                        ]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "typing.Optional[int]".to_string(),
                source_module: None,
                import: ::std::collections::HashSet::from([
                    "typing".into(),
                    "collections.abc".into(),
                ]),
                type_refs: ::std::collections::HashMap::new(),
            },
            doc: "Process a callback function",
            module: None,
            is_async: false,
            deprecated: None,
            type_ignored: None,
            is_overload: false,
            file: file!(),
            line: line!(),
            column: column!(),
            index: 0usize,
        }
        "#);
        Ok(())
    }

    #[test]
    fn test_complex_types() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            import collections.abc
            import typing

            def fn_override_type(cb: collections.abc.Callable[[str], typing.Any]) -> collections.abc.Callable[[str], typing.Any]:
                """Example function with complex types"""
            "#
        })?;
        let info = parse_python_function_stub(stub_str)?;
        let out = info.to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r#"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "fn_override_type",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "cb",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "collections.abc.Callable[[str], typing.Any]".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from([
                            "collections.abc".into(),
                            "typing".into(),
                        ]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "collections.abc.Callable[[str], typing.Any]".to_string(),
                source_module: None,
                import: ::std::collections::HashSet::from([
                    "collections.abc".into(),
                    "typing".into(),
                ]),
                type_refs: ::std::collections::HashMap::new(),
            },
            doc: "Example function with complex types",
            module: None,
            is_async: false,
            deprecated: None,
            type_ignored: None,
            is_overload: false,
            file: file!(),
            line: line!(),
            column: column!(),
            index: 0usize,
        }
        "#);
        Ok(())
    }

    #[test]
    fn test_multiple_args() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            import typing

            def add(a: int, b: int, c: typing.Optional[int]) -> int: ...
            "#
        })?;
        let info = parse_python_function_stub(stub_str)?;
        let out = info.to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r#"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "add",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "a",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "int".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from(["typing".into()]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "b",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "int".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from(["typing".into()]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "c",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "typing.Optional[int]".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from(["typing".into()]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "int".to_string(),
                source_module: None,
                import: ::std::collections::HashSet::from(["typing".into()]),
                type_refs: ::std::collections::HashMap::new(),
            },
            doc: "",
            module: None,
            is_async: false,
            deprecated: None,
            type_ignored: None,
            is_overload: false,
            file: file!(),
            line: line!(),
            column: column!(),
            index: 0usize,
        }
        "#);
        Ok(())
    }

    #[test]
    fn test_no_return_type() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            def print_hello(name: str):
                """Print a greeting"""
            "#
        })?;
        let info = parse_python_function_stub(stub_str)?;
        let out = info.to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r#"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "print_hello",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "name",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "str".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from([]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
            ],
            r#return: ::pyo3_stub_gen::type_info::no_return_type_output,
            doc: "Print a greeting",
            module: None,
            is_async: false,
            deprecated: None,
            type_ignored: None,
            is_overload: false,
            file: file!(),
            line: line!(),
            column: column!(),
            index: 0usize,
        }
        "#);
        Ok(())
    }

    #[test]
    fn test_async_function() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            async def fetch_data(url: str) -> str:
                """Fetch data from URL"""
            "#
        })?;
        let info = parse_python_function_stub(stub_str)?;
        let out = info.to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r#"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "fetch_data",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "url",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "str".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from([]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "str".to_string(),
                source_module: None,
                import: ::std::collections::HashSet::from([]),
                type_refs: ::std::collections::HashMap::new(),
            },
            doc: "Fetch data from URL",
            module: None,
            is_async: true,
            deprecated: None,
            type_ignored: None,
            is_overload: false,
            file: file!(),
            line: line!(),
            column: column!(),
            index: 0usize,
        }
        "#);
        Ok(())
    }

    #[test]
    fn test_deprecated_decorator() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            @deprecated
            def old_function(x: int) -> int:
                """This function is deprecated"""
            "#
        })?;
        let info = parse_python_function_stub(stub_str)?;
        let out = info.to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r#"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "old_function",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "x",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "int".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from([]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "int".to_string(),
                source_module: None,
                import: ::std::collections::HashSet::from([]),
                type_refs: ::std::collections::HashMap::new(),
            },
            doc: "This function is deprecated",
            module: None,
            is_async: false,
            deprecated: Some(::pyo3_stub_gen::type_info::DeprecatedInfo {
                since: None,
                note: None,
            }),
            type_ignored: None,
            is_overload: false,
            file: file!(),
            line: line!(),
            column: column!(),
            index: 0usize,
        }
        "#);
        Ok(())
    }

    #[test]
    fn test_deprecated_with_message() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            @deprecated("Use new_function instead")
            def old_function(x: int) -> int:
                """This function is deprecated"""
            "#
        })?;
        let info = parse_python_function_stub(stub_str)?;
        let out = info.to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r#"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "old_function",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "x",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "int".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from([]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "int".to_string(),
                source_module: None,
                import: ::std::collections::HashSet::from([]),
                type_refs: ::std::collections::HashMap::new(),
            },
            doc: "This function is deprecated",
            module: None,
            is_async: false,
            deprecated: Some(::pyo3_stub_gen::type_info::DeprecatedInfo {
                since: None,
                note: Some("Use new_function instead"),
            }),
            type_ignored: None,
            is_overload: false,
            file: file!(),
            line: line!(),
            column: column!(),
            index: 0usize,
        }
        "#);
        Ok(())
    }

    #[test]
    fn test_rust_type_marker() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            def process_data(x: pyo3_stub_gen.RustType["MyRustType"]) -> pyo3_stub_gen.RustType["MyRustType"]:
                """Process data using Rust type marker"""
            "#
        })?;
        let info = parse_python_function_stub(stub_str)?;
        let out = info.to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "process_data",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "x",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: <MyRustType as ::pyo3_stub_gen::PyStubType>::type_input,
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
            ],
            r#return: <MyRustType as pyo3_stub_gen::PyStubType>::type_output,
            doc: "Process data using Rust type marker",
            module: None,
            is_async: false,
            deprecated: None,
            type_ignored: None,
            is_overload: false,
            file: file!(),
            line: line!(),
            column: column!(),
            index: 0usize,
        }
        "###);
        Ok(())
    }

    #[test]
    fn test_rust_type_marker_with_path() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            def process(x: pyo3_stub_gen.RustType["crate::MyType"]) -> pyo3_stub_gen.RustType["Vec<String>"]:
                """Test with type paths"""
            "#
        })?;
        let info = parse_python_function_stub(stub_str)?;
        let out = info.to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "process",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "x",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: <crate::MyType as ::pyo3_stub_gen::PyStubType>::type_input,
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
            ],
            r#return: <Vec<String> as pyo3_stub_gen::PyStubType>::type_output,
            doc: "Test with type paths",
            module: None,
            is_async: false,
            deprecated: None,
            type_ignored: None,
            is_overload: false,
            file: file!(),
            line: line!(),
            column: column!(),
            index: 0usize,
        }
        "###);
        Ok(())
    }

    #[test]
    fn test_keyword_only_args() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            import typing

            def configure(name: str, *, dtype: str, ndim: int, jagged: bool = False) -> None:
                """Test keyword-only parameters"""
            "#
        })?;
        let info = parse_python_function_stub(stub_str)?;
        let out = info.to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r#"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "configure",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "name",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "str".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from(["typing".into()]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "dtype",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::KeywordOnly,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "str".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from(["typing".into()]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "ndim",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::KeywordOnly,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "int".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from(["typing".into()]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "jagged",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::KeywordOnly,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "bool".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from(["typing".into()]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::Expr({
                        fn _fmt() -> String {
                            "False".to_string()
                        }
                        _fmt
                    }),
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "None".to_string(),
                source_module: None,
                import: ::std::collections::HashSet::from(["typing".into()]),
                type_refs: ::std::collections::HashMap::new(),
            },
            doc: "Test keyword-only parameters",
            module: None,
            is_async: false,
            deprecated: None,
            type_ignored: None,
            is_overload: false,
            file: file!(),
            line: line!(),
            column: column!(),
            index: 0usize,
        }
        "#);
        Ok(())
    }

    #[test]
    fn test_positional_only_args() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            def func(x: int, y: int, /, z: int) -> int:
                """Test positional-only parameters"""
            "#
        })?;
        let info = parse_python_function_stub(stub_str)?;
        let out = info.to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r#"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "func",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "x",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOnly,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "int".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from([]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "y",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOnly,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "int".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from([]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "z",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "int".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from([]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "int".to_string(),
                source_module: None,
                import: ::std::collections::HashSet::from([]),
                type_refs: ::std::collections::HashMap::new(),
            },
            doc: "Test positional-only parameters",
            module: None,
            is_async: false,
            deprecated: None,
            type_ignored: None,
            is_overload: false,
            file: file!(),
            line: line!(),
            column: column!(),
            index: 0usize,
        }
        "#);
        Ok(())
    }

    #[test]
    fn test_single_overload() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            @overload
            def foo(x: int) -> int:
                """Integer overload"""
            "#
        })?;
        let info = parse_python_function_stub(stub_str)?;
        let out = info.to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r#"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "foo",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "x",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "int".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from([]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "int".to_string(),
                source_module: None,
                import: ::std::collections::HashSet::from([]),
                type_refs: ::std::collections::HashMap::new(),
            },
            doc: "Integer overload",
            module: None,
            is_async: false,
            deprecated: None,
            type_ignored: None,
            is_overload: true,
            file: file!(),
            line: line!(),
            column: column!(),
            index: 0usize,
        }
        "#);
        Ok(())
    }

    #[test]
    fn test_multiple_overloads() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            @overload
            def foo(x: int) -> int:
                """Integer overload"""

            @overload
            def foo(x: float) -> float:
                """Float overload"""
            "#
        })?;
        let infos = parse_python_overload_stubs(stub_str, "foo")?;
        assert_eq!(infos.len(), 2);

        let out1 = infos[0].to_token_stream();
        insta::assert_snapshot!(format_as_value(out1), @r#"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "foo",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "x",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "int".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from([]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "int".to_string(),
                source_module: None,
                import: ::std::collections::HashSet::from([]),
                type_refs: ::std::collections::HashMap::new(),
            },
            doc: "Integer overload",
            module: None,
            is_async: false,
            deprecated: None,
            type_ignored: None,
            is_overload: true,
            file: file!(),
            line: line!(),
            column: column!(),
            index: 0usize,
        }
        "#);

        let out2 = infos[1].to_token_stream();
        insta::assert_snapshot!(format_as_value(out2), @r#"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "foo",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "x",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "float".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from([]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "float".to_string(),
                source_module: None,
                import: ::std::collections::HashSet::from([]),
                type_refs: ::std::collections::HashMap::new(),
            },
            doc: "Float overload",
            module: None,
            is_async: false,
            deprecated: None,
            type_ignored: None,
            is_overload: true,
            file: file!(),
            line: line!(),
            column: column!(),
            index: 0usize,
        }
        "#);
        Ok(())
    }

    #[test]
    fn test_overload_with_literal_types() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            import typing
            @overload
            def as_tuple(xs: list[int], *, tuple_out: typing.Literal[True]) -> tuple[int, ...]:
                """Return as tuple"""
            "#
        })?;
        let info = parse_python_function_stub(stub_str)?;
        let out = info.to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r#"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "as_tuple",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "xs",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "list[int]".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from(["typing".into()]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "tuple_out",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::KeywordOnly,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "typing.Literal[True]".to_string(),
                        source_module: None,
                        import: ::std::collections::HashSet::from(["typing".into()]),
                        type_refs: ::std::collections::HashMap::new(),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "tuple[int, ...]".to_string(),
                source_module: None,
                import: ::std::collections::HashSet::from(["typing".into()]),
                type_refs: ::std::collections::HashMap::new(),
            },
            doc: "Return as tuple",
            module: None,
            is_async: false,
            deprecated: None,
            type_ignored: None,
            is_overload: true,
            file: file!(),
            line: line!(),
            column: column!(),
            index: 0usize,
        }
        "#);
        Ok(())
    }

    fn format_as_value(tt: TokenStream2) -> String {
        let ttt = quote! { const _: () = #tt; };
        let formatted = prettyplease::unparse(&syn::parse_file(&ttt.to_string()).unwrap());
        formatted
            .trim()
            .strip_prefix("const _: () = ")
            .unwrap()
            .strip_suffix(';')
            .unwrap()
            .to_string()
    }
}
