use indexmap::IndexSet;
use rustpython_parser::{ast, Parse};
use syn::{Error, LitStr, Result, Type};

use super::{
    arg::ArgInfo, attr::DeprecatedInfo, method::MethodInfo, method::MethodType,
    pyfunction::PyFunctionInfo, util::TypeOrOverride,
};

/// Remove common leading whitespace from all lines (similar to Python's textwrap.dedent)
fn dedent(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();

    // Find the minimum indentation (ignoring empty lines)
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    // Remove the common indentation from each line
    lines
        .iter()
        .map(|line| {
            if line.len() >= min_indent {
                &line[min_indent..]
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
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

    // Generate PyFunctionInfo
    build_py_function_info(&func_def, &imports, is_async)
}

/// Parse Python class definition and return class name and methods
/// Returns (class_name, Vec<MethodInfo>)
pub fn parse_python_class_methods(input: &LitStr) -> Result<(String, Vec<MethodInfo>)> {
    let stub_content = input.value();

    // Remove common indentation to allow indented Python code in raw strings
    let dedented_content = dedent(&stub_content);

    // Parse Python code using rustpython-parser
    let parsed = ast::Suite::parse(&dedented_content, "<stub>")
        .map_err(|e| Error::new(input.span(), format!("Failed to parse Python stub: {}", e)))?;

    // Extract imports and class definition
    let mut imports = Vec::new();
    let mut class_def: Option<ast::StmtClassDef> = None;

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
            ast::Stmt::ClassDef(cls_def) => {
                if class_def.is_some() {
                    return Err(Error::new(
                        input.span(),
                        "Multiple class definitions found. Only one class is allowed per gen_methods_from_python! call",
                    ));
                }
                class_def = Some(cls_def);
            }
            _ => {
                // Ignore other statements
            }
        }
    }

    // Check that exactly one class is defined
    let class_def = class_def
        .ok_or_else(|| Error::new(input.span(), "No class definition found in Python stub"))?;

    let class_name = class_def.name.to_string();

    // Extract methods from class body
    let mut methods = Vec::new();
    for stmt in &class_def.body {
        match stmt {
            ast::Stmt::FunctionDef(func_def) => {
                let method = build_method_info(func_def, &imports, false)?;
                methods.push(method);
            }
            ast::Stmt::AsyncFunctionDef(func_def) => {
                // Convert AsyncFunctionDef to FunctionDef for uniform processing
                let sync_func = ast::StmtFunctionDef {
                    range: func_def.range.clone(),
                    name: func_def.name.clone(),
                    type_params: func_def.type_params.clone(),
                    args: func_def.args.clone(),
                    body: func_def.body.clone(),
                    decorator_list: func_def.decorator_list.clone(),
                    returns: func_def.returns.clone(),
                    type_comment: func_def.type_comment.clone(),
                };
                let method = build_method_info(&sync_func, &imports, true)?;
                methods.push(method);
            }
            _ => {
                // Ignore other statements (e.g., docstrings, pass)
            }
        }
    }

    if methods.is_empty() {
        return Err(Error::new(
            input.span(),
            "No method definitions found in class body",
        ));
    }

    Ok((class_name, methods))
}

/// Build PyFunctionInfo from Python function definition
fn build_py_function_info(
    func_def: &ast::StmtFunctionDef,
    imports: &[String],
    is_async: bool,
) -> Result<PyFunctionInfo> {
    let func_name = func_def.name.to_string();

    // Extract docstring
    let doc = extract_docstring(func_def);

    // Extract arguments
    let args = extract_args(&func_def.args, imports)?;

    // Extract return type
    let return_type = extract_return_type(&func_def.returns, imports)?;

    // Try to extract deprecated decorator
    let deprecated = extract_deprecated_from_decorators(&func_def.decorator_list);

    // Note: type_ignored (# type: ignore comments) cannot be extracted from Python AST
    // as rustpython-parser doesn't preserve comments

    // Construct PyFunctionInfo
    Ok(PyFunctionInfo {
        name: func_name,
        args,
        r#return: return_type,
        sig: None,
        doc,
        module: None,
        is_async,
        deprecated,
        type_ignored: None,
    })
}

/// Build MethodInfo from Python function definition
fn build_method_info(
    func_def: &ast::StmtFunctionDef,
    imports: &[String],
    is_async: bool,
) -> Result<MethodInfo> {
    let func_name = func_def.name.to_string();

    // Extract docstring
    let doc = extract_docstring(func_def);

    // Determine method type based on decorators and function name
    let method_type = determine_method_type(func_def, &func_def.args);

    // Extract arguments (skip 'self' or 'cls' for instance/class methods)
    let args = extract_args_for_method(&func_def.args, imports, method_type)?;

    // Extract return type
    let return_type = extract_return_type(&func_def.returns, imports)?;

    // Try to extract deprecated decorator
    let deprecated = extract_deprecated_from_decorators(&func_def.decorator_list);

    // Construct MethodInfo
    Ok(MethodInfo {
        name: func_name,
        args,
        sig: None,
        r#return: return_type,
        doc,
        r#type: method_type,
        is_async,
        deprecated,
        type_ignored: None,
    })
}

/// Determine method type from decorators and arguments
fn determine_method_type(func_def: &ast::StmtFunctionDef, args: &ast::Arguments) -> MethodType {
    // Check for @staticmethod decorator
    for decorator in &func_def.decorator_list {
        if let ast::Expr::Name(name) = decorator {
            match name.id.as_str() {
                "staticmethod" => return MethodType::Static,
                "classmethod" => return MethodType::Class,
                _ => {}
            }
        }
    }

    // Check if it's __new__ (constructor)
    if func_def.name.as_str() == "__new__" {
        return MethodType::New;
    }

    // Check first argument to determine if it's instance/class method
    if let Some(first_arg) = args.args.first() {
        let arg_name = first_arg.def.arg.as_str();
        if arg_name == "self" {
            return MethodType::Instance;
        } else if arg_name == "cls" {
            return MethodType::Class;
        }
    }

    // Default to instance method
    MethodType::Instance
}

/// Extract arguments for method (handling self/cls)
fn extract_args_for_method(
    args: &ast::Arguments,
    imports: &[String],
    method_type: MethodType,
) -> Result<Vec<ArgInfo>> {
    let mut arg_infos = Vec::new();

    // Dummy type for TypeOrOverride (not used in ToTokens for OverrideType)
    let dummy_type: Type = syn::parse_str("()").unwrap();

    // Process positional arguments
    for (idx, arg) in args.args.iter().enumerate() {
        let arg_name = arg.def.arg.to_string();

        // Skip 'self' or 'cls' for instance/class/new methods (first argument only)
        if idx == 0 {
            if (method_type == MethodType::Instance && arg_name == "self")
                || (method_type == MethodType::Class && arg_name == "cls")
                || (method_type == MethodType::New && arg_name == "cls")
            {
                continue;
            }
        }

        let type_override = if let Some(annotation) = &arg.def.annotation {
            type_annotation_to_type_override(annotation, imports, dummy_type.clone())?
        } else {
            // No type annotation - use Any
            TypeOrOverride::OverrideType {
                r#type: dummy_type.clone(),
                type_repr: "typing.Any".to_string(),
                imports: IndexSet::from(["typing".to_string()]),
            }
        };

        arg_infos.push(ArgInfo {
            name: arg_name,
            r#type: type_override,
        });
    }

    Ok(arg_infos)
}

/// Extract docstring from function definition
fn extract_docstring(func_def: &ast::StmtFunctionDef) -> String {
    if let Some(ast::Stmt::Expr(expr_stmt)) = func_def.body.first() {
        if let ast::Expr::Constant(constant) = &*expr_stmt.value {
            if let ast::Constant::Str(s) = &constant.value {
                return s.to_string();
            }
        }
    }
    String::new()
}

/// Extract deprecated decorator information if present
fn extract_deprecated_from_decorators(decorators: &[ast::Expr]) -> Option<DeprecatedInfo> {
    for decorator in decorators {
        // Check for @deprecated or @deprecated("message")
        match decorator {
            ast::Expr::Name(name) if name.id.as_str() == "deprecated" => {
                return Some(DeprecatedInfo {
                    since: None,
                    note: None,
                });
            }
            ast::Expr::Call(call) => {
                if let ast::Expr::Name(name) = &*call.func {
                    if name.id.as_str() == "deprecated" {
                        // Try to extract the message from the first argument
                        let note = call.args.first().and_then(|arg| match arg {
                            ast::Expr::Constant(constant) => match &constant.value {
                                ast::Constant::Str(s) => Some(s.to_string()),
                                _ => None,
                            },
                            _ => None,
                        });
                        return Some(DeprecatedInfo { since: None, note });
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// Extract arguments from function definition
fn extract_args(args: &ast::Arguments, imports: &[String]) -> Result<Vec<ArgInfo>> {
    let mut arg_infos = Vec::new();

    // Dummy type for TypeOrOverride (not used in ToTokens for OverrideType)
    let dummy_type: Type = syn::parse_str("()").unwrap();

    // Process positional arguments
    for arg in &args.args {
        let arg_name = arg.def.arg.to_string();

        // Skip 'self' argument
        if arg_name == "self" {
            continue;
        }

        let type_override = if let Some(annotation) = &arg.def.annotation {
            type_annotation_to_type_override(annotation, imports, dummy_type.clone())?
        } else {
            // No type annotation - use Any
            TypeOrOverride::OverrideType {
                r#type: dummy_type.clone(),
                type_repr: "typing.Any".to_string(),
                imports: IndexSet::from(["typing".to_string()]),
            }
        };

        arg_infos.push(ArgInfo {
            name: arg_name,
            r#type: type_override,
        });
    }

    Ok(arg_infos)
}

/// Extract return type from function definition
fn extract_return_type(
    returns: &Option<Box<ast::Expr>>,
    imports: &[String],
) -> Result<Option<TypeOrOverride>> {
    // Dummy type for TypeOrOverride (not used in ToTokens for OverrideType)
    let dummy_type: Type = syn::parse_str("()").unwrap();

    if let Some(return_annotation) = returns {
        Ok(Some(type_annotation_to_type_override(
            return_annotation,
            imports,
            dummy_type,
        )?))
    } else {
        // No return type annotation - use None (void)
        Ok(None)
    }
}

/// Convert Python type annotation to TypeOrOverride
fn type_annotation_to_type_override(
    expr: &ast::Expr,
    imports: &[String],
    dummy_type: Type,
) -> Result<TypeOrOverride> {
    let type_str = expr_to_type_string(expr);

    // Convert imports to IndexSet
    let import_set: IndexSet<String> = imports.iter().map(|s| s.to_string()).collect();

    Ok(TypeOrOverride::OverrideType {
        r#type: dummy_type,
        type_repr: type_str,
        imports: import_set,
    })
}

/// Convert Python expression to type string
fn expr_to_type_string(expr: &ast::Expr) -> String {
    expr_to_type_string_inner(expr, false)
}

/// Convert Python expression to type string with context
fn expr_to_type_string_inner(expr: &ast::Expr, in_subscript: bool) -> String {
    match expr {
        ast::Expr::Name(name) => name.id.to_string(),
        ast::Expr::Attribute(attr) => {
            format!(
                "{}.{}",
                expr_to_type_string_inner(&attr.value, false),
                attr.attr
            )
        }
        ast::Expr::Subscript(subscript) => {
            let base = expr_to_type_string_inner(&subscript.value, false);
            let slice = expr_to_type_string_inner(&subscript.slice, true);
            format!("{}[{}]", base, slice)
        }
        ast::Expr::List(list) => {
            let elements: Vec<String> = list
                .elts
                .iter()
                .map(|e| expr_to_type_string_inner(e, false))
                .collect();
            format!("[{}]", elements.join(", "))
        }
        ast::Expr::Tuple(tuple) => {
            let elements: Vec<String> = tuple
                .elts
                .iter()
                .map(|e| expr_to_type_string_inner(e, in_subscript))
                .collect();
            if in_subscript {
                // In subscript context, preserve tuple structure without extra parentheses
                elements.join(", ")
            } else {
                format!("({})", elements.join(", "))
            }
        }
        ast::Expr::Constant(constant) => match &constant.value {
            ast::Constant::Int(i) => i.to_string(),
            ast::Constant::Str(s) => format!("\"{}\"", s),
            ast::Constant::Bool(b) => b.to_string(),
            ast::Constant::None => "None".to_string(),
            _ => "Any".to_string(),
        },
        _ => "Any".to_string(),
    }
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
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "foo",
            args: &[
                ::pyo3_stub_gen::type_info::ArgInfo {
                    name: "x",
                    r#type: || ::pyo3_stub_gen::TypeInfo {
                        name: "int".to_string(),
                        import: ::std::collections::HashSet::from([]),
                    },
                    signature: None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "int".to_string(),
                import: ::std::collections::HashSet::from([]),
            },
            doc: "A simple function",
            module: None,
            is_async: false,
            deprecated: None,
            type_ignored: None,
        }
        "###);
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
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "process",
            args: &[
                ::pyo3_stub_gen::type_info::ArgInfo {
                    name: "func",
                    r#type: || ::pyo3_stub_gen::TypeInfo {
                        name: "Callable[[str], int]".to_string(),
                        import: ::std::collections::HashSet::from([
                            "typing".into(),
                            "collections.abc".into(),
                        ]),
                    },
                    signature: None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "typing.Optional[int]".to_string(),
                import: ::std::collections::HashSet::from([
                    "typing".into(),
                    "collections.abc".into(),
                ]),
            },
            doc: "Process a callback function",
            module: None,
            is_async: false,
            deprecated: None,
            type_ignored: None,
        }
        "###);
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
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "fn_override_type",
            args: &[
                ::pyo3_stub_gen::type_info::ArgInfo {
                    name: "cb",
                    r#type: || ::pyo3_stub_gen::TypeInfo {
                        name: "collections.abc.Callable[[str], typing.Any]".to_string(),
                        import: ::std::collections::HashSet::from([
                            "collections.abc".into(),
                            "typing".into(),
                        ]),
                    },
                    signature: None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "collections.abc.Callable[[str], typing.Any]".to_string(),
                import: ::std::collections::HashSet::from([
                    "collections.abc".into(),
                    "typing".into(),
                ]),
            },
            doc: "Example function with complex types",
            module: None,
            is_async: false,
            deprecated: None,
            type_ignored: None,
        }
        "###);
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
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "add",
            args: &[
                ::pyo3_stub_gen::type_info::ArgInfo {
                    name: "a",
                    r#type: || ::pyo3_stub_gen::TypeInfo {
                        name: "int".to_string(),
                        import: ::std::collections::HashSet::from(["typing".into()]),
                    },
                    signature: None,
                },
                ::pyo3_stub_gen::type_info::ArgInfo {
                    name: "b",
                    r#type: || ::pyo3_stub_gen::TypeInfo {
                        name: "int".to_string(),
                        import: ::std::collections::HashSet::from(["typing".into()]),
                    },
                    signature: None,
                },
                ::pyo3_stub_gen::type_info::ArgInfo {
                    name: "c",
                    r#type: || ::pyo3_stub_gen::TypeInfo {
                        name: "typing.Optional[int]".to_string(),
                        import: ::std::collections::HashSet::from(["typing".into()]),
                    },
                    signature: None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "int".to_string(),
                import: ::std::collections::HashSet::from(["typing".into()]),
            },
            doc: "",
            module: None,
            is_async: false,
            deprecated: None,
            type_ignored: None,
        }
        "###);
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
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "print_hello",
            args: &[
                ::pyo3_stub_gen::type_info::ArgInfo {
                    name: "name",
                    r#type: || ::pyo3_stub_gen::TypeInfo {
                        name: "str".to_string(),
                        import: ::std::collections::HashSet::from([]),
                    },
                    signature: None,
                },
            ],
            r#return: ::pyo3_stub_gen::type_info::no_return_type_output,
            doc: "Print a greeting",
            module: None,
            is_async: false,
            deprecated: None,
            type_ignored: None,
        }
        "###);
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
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "fetch_data",
            args: &[
                ::pyo3_stub_gen::type_info::ArgInfo {
                    name: "url",
                    r#type: || ::pyo3_stub_gen::TypeInfo {
                        name: "str".to_string(),
                        import: ::std::collections::HashSet::from([]),
                    },
                    signature: None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "str".to_string(),
                import: ::std::collections::HashSet::from([]),
            },
            doc: "Fetch data from URL",
            module: None,
            is_async: true,
            deprecated: None,
            type_ignored: None,
        }
        "###);
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
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "old_function",
            args: &[
                ::pyo3_stub_gen::type_info::ArgInfo {
                    name: "x",
                    r#type: || ::pyo3_stub_gen::TypeInfo {
                        name: "int".to_string(),
                        import: ::std::collections::HashSet::from([]),
                    },
                    signature: None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "int".to_string(),
                import: ::std::collections::HashSet::from([]),
            },
            doc: "This function is deprecated",
            module: None,
            is_async: false,
            deprecated: Some(::pyo3_stub_gen::type_info::DeprecatedInfo {
                since: None,
                note: None,
            }),
            type_ignored: None,
        }
        "###);
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
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: "old_function",
            args: &[
                ::pyo3_stub_gen::type_info::ArgInfo {
                    name: "x",
                    r#type: || ::pyo3_stub_gen::TypeInfo {
                        name: "int".to_string(),
                        import: ::std::collections::HashSet::from([]),
                    },
                    signature: None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "int".to_string(),
                import: ::std::collections::HashSet::from([]),
            },
            doc: "This function is deprecated",
            module: None,
            is_async: false,
            deprecated: Some(::pyo3_stub_gen::type_info::DeprecatedInfo {
                since: None,
                note: Some("Use new_function instead"),
            }),
            type_ignored: None,
        }
        "###);
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

    // Tests for class methods parsing
    #[test]
    fn test_single_method_class() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            class Incrementer:
                def increment(self, x: int) -> int:
                    """Increment by one"""
            "#
        })?;
        let (class_name, methods) = parse_python_class_methods(&stub_str)?;
        assert_eq!(class_name, "Incrementer");
        assert_eq!(methods.len(), 1);

        let out = methods[0].to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::MethodInfo {
            name: "increment",
            args: &[
                ::pyo3_stub_gen::type_info::ArgInfo {
                    name: "x",
                    r#type: || ::pyo3_stub_gen::TypeInfo {
                        name: "int".to_string(),
                        import: ::std::collections::HashSet::from([]),
                    },
                    signature: None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "int".to_string(),
                import: ::std::collections::HashSet::from([]),
            },
            doc: "Increment by one",
            r#type: ::pyo3_stub_gen::type_info::MethodType::Instance,
            is_async: false,
            deprecated: None,
            type_ignored: None,
        }
        "###);
        Ok(())
    }

    #[test]
    fn test_multiple_methods_class() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            class Incrementer:
                def increment_1(self, x: int) -> int:
                    """First method"""

                def increment_2(self, x: float) -> float:
                    """Second method"""
            "#
        })?;
        let (class_name, methods) = parse_python_class_methods(&stub_str)?;
        assert_eq!(class_name, "Incrementer");
        assert_eq!(methods.len(), 2);

        assert_eq!(methods[0].name, "increment_1");
        assert_eq!(methods[1].name, "increment_2");
        Ok(())
    }

    #[test]
    fn test_static_method_in_class() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            class MyClass:
                @staticmethod
                def create(name: str) -> str:
                    """Create something"""
            "#
        })?;
        let (class_name, methods) = parse_python_class_methods(&stub_str)?;
        assert_eq!(class_name, "MyClass");
        assert_eq!(methods.len(), 1);

        let out = methods[0].to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::MethodInfo {
            name: "create",
            args: &[
                ::pyo3_stub_gen::type_info::ArgInfo {
                    name: "name",
                    r#type: || ::pyo3_stub_gen::TypeInfo {
                        name: "str".to_string(),
                        import: ::std::collections::HashSet::from([]),
                    },
                    signature: None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "str".to_string(),
                import: ::std::collections::HashSet::from([]),
            },
            doc: "Create something",
            r#type: ::pyo3_stub_gen::type_info::MethodType::Static,
            is_async: false,
            deprecated: None,
            type_ignored: None,
        }
        "###);
        Ok(())
    }

    #[test]
    fn test_class_method_in_class() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            class MyClass:
                @classmethod
                def from_string(cls, s: str) -> int:
                    """Create from string"""
            "#
        })?;
        let (class_name, methods) = parse_python_class_methods(&stub_str)?;
        assert_eq!(class_name, "MyClass");
        assert_eq!(methods.len(), 1);

        let out = methods[0].to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::MethodInfo {
            name: "from_string",
            args: &[
                ::pyo3_stub_gen::type_info::ArgInfo {
                    name: "s",
                    r#type: || ::pyo3_stub_gen::TypeInfo {
                        name: "str".to_string(),
                        import: ::std::collections::HashSet::from([]),
                    },
                    signature: None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "int".to_string(),
                import: ::std::collections::HashSet::from([]),
            },
            doc: "Create from string",
            r#type: ::pyo3_stub_gen::type_info::MethodType::Class,
            is_async: false,
            deprecated: None,
            type_ignored: None,
        }
        "###);
        Ok(())
    }

    #[test]
    fn test_new_method_in_class() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            class MyClass:
                def __new__(cls) -> object:
                    """Constructor"""
            "#
        })?;
        let (class_name, methods) = parse_python_class_methods(&stub_str)?;
        assert_eq!(class_name, "MyClass");
        assert_eq!(methods.len(), 1);

        let out = methods[0].to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::MethodInfo {
            name: "__new__",
            args: &[],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "object".to_string(),
                import: ::std::collections::HashSet::from([]),
            },
            doc: "Constructor",
            r#type: ::pyo3_stub_gen::type_info::MethodType::New,
            is_async: false,
            deprecated: None,
            type_ignored: None,
        }
        "###);
        Ok(())
    }

    #[test]
    fn test_method_with_imports_in_class() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            import typing
            from collections.abc import Callable

            class MyClass:
                def process(self, func: Callable[[str], int]) -> typing.Optional[int]:
                    """Process a callback"""
            "#
        })?;
        let (class_name, methods) = parse_python_class_methods(&stub_str)?;
        assert_eq!(class_name, "MyClass");
        assert_eq!(methods.len(), 1);

        let out = methods[0].to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::MethodInfo {
            name: "process",
            args: &[
                ::pyo3_stub_gen::type_info::ArgInfo {
                    name: "func",
                    r#type: || ::pyo3_stub_gen::TypeInfo {
                        name: "Callable[[str], int]".to_string(),
                        import: ::std::collections::HashSet::from([
                            "typing".into(),
                            "collections.abc".into(),
                        ]),
                    },
                    signature: None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "typing.Optional[int]".to_string(),
                import: ::std::collections::HashSet::from([
                    "typing".into(),
                    "collections.abc".into(),
                ]),
            },
            doc: "Process a callback",
            r#type: ::pyo3_stub_gen::type_info::MethodType::Instance,
            is_async: false,
            deprecated: None,
            type_ignored: None,
        }
        "###);
        Ok(())
    }

    #[test]
    fn test_async_method_in_class() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            class MyClass:
                async def fetch_data(self, url: str) -> str:
                    """Fetch data asynchronously"""
            "#
        })?;
        let (class_name, methods) = parse_python_class_methods(&stub_str)?;
        assert_eq!(class_name, "MyClass");
        assert_eq!(methods.len(), 1);

        let out = methods[0].to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::MethodInfo {
            name: "fetch_data",
            args: &[
                ::pyo3_stub_gen::type_info::ArgInfo {
                    name: "url",
                    r#type: || ::pyo3_stub_gen::TypeInfo {
                        name: "str".to_string(),
                        import: ::std::collections::HashSet::from([]),
                    },
                    signature: None,
                },
            ],
            r#return: || ::pyo3_stub_gen::TypeInfo {
                name: "str".to_string(),
                import: ::std::collections::HashSet::from([]),
            },
            doc: "Fetch data asynchronously",
            r#type: ::pyo3_stub_gen::type_info::MethodType::Instance,
            is_async: true,
            deprecated: None,
            type_ignored: None,
        }
        "###);
        Ok(())
    }
}
