use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rustpython_parser::{ast, Parse};
use syn::{Error, LitStr, Result};

/// Parse Python stub string and generate PyFunctionInfo token stream
pub fn gen_function_from_python_impl(input: TokenStream2) -> Result<TokenStream2> {
    // Parse the input as a string literal
    let stub_str: LitStr = syn::parse2(input)?;
    let stub_content = stub_str.value();

    // Parse Python code using rustpython-parser
    let parsed = ast::Suite::parse(&stub_content, "<stub>")
        .map_err(|e| Error::new(stub_str.span(), format!("Failed to parse Python stub: {}", e)))?;

    // Extract imports and function definitions
    let mut imports = Vec::new();
    let mut functions = Vec::new();

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
                functions.push(func_def);
            }
            _ => {
                // Ignore other statements
            }
        }
    }

    // Check that exactly one function is defined
    if functions.is_empty() {
        return Err(Error::new(
            stub_str.span(),
            "No function definition found in Python stub",
        ));
    }
    if functions.len() > 1 {
        return Err(Error::new(
            stub_str.span(),
            "Multiple function definitions found. Only one function is allowed per gen_function_from_python! call",
        ));
    }

    let func_def = &functions[0];

    // Generate PyFunctionInfo token stream
    generate_py_function_info(func_def, &imports)
}

/// Generate PyFunctionInfo token stream from Python function definition
fn generate_py_function_info(
    func_def: &ast::StmtFunctionDef,
    imports: &[String],
) -> Result<TokenStream2> {
    let func_name = func_def.name.to_string();

    // Extract docstring
    let doc = extract_docstring(func_def);

    // Extract arguments
    let args = extract_args(&func_def.args, imports)?;

    // Extract return type
    let return_type = extract_return_type(&func_def.returns, imports)?;

    // Check if function is async
    let is_async = false; // TODO: handle async functions

    // Generate token stream
    Ok(quote! {
        ::pyo3_stub_gen::type_info::PyFunctionInfo {
            name: #func_name,
            args: &[#(#args),*],
            r#return: #return_type,
            doc: #doc,
            module: None,
            is_async: #is_async,
            deprecated: None,
            type_ignored: None,
        }
    })
}

/// Extract docstring from function definition
fn extract_docstring(func_def: &ast::StmtFunctionDef) -> String {
    if let Some(first_stmt) = func_def.body.first() {
        if let ast::Stmt::Expr(expr_stmt) = first_stmt {
            if let ast::Expr::Constant(constant) = &*expr_stmt.value {
                if let ast::Constant::Str(s) = &constant.value {
                    return s.to_string();
                }
            }
        }
    }
    String::new()
}

/// Extract arguments from function definition
fn extract_args(
    args: &ast::Arguments,
    imports: &[String],
) -> Result<Vec<TokenStream2>> {
    let mut arg_tokens = Vec::new();

    // Process positional arguments
    for arg in &args.args {
        let arg_name = arg.def.arg.to_string();

        // Skip 'self' argument
        if arg_name == "self" {
            continue;
        }

        let type_info = if let Some(annotation) = &arg.def.annotation {
            type_annotation_to_token_stream(annotation, imports)?
        } else {
            // No type annotation - use Any
            quote! {
                || ::pyo3_stub_gen::TypeInfo {
                    name: "typing.Any".to_string(),
                    import: ::std::collections::HashSet::from(["typing".into()])
                }
            }
        };

        arg_tokens.push(quote! {
            ::pyo3_stub_gen::type_info::ArgInfo {
                name: #arg_name,
                r#type: #type_info,
                signature: None,
            }
        });
    }

    Ok(arg_tokens)
}

/// Extract return type from function definition
fn extract_return_type(
    returns: &Option<Box<ast::Expr>>,
    imports: &[String],
) -> Result<TokenStream2> {
    if let Some(return_annotation) = returns {
        type_annotation_to_token_stream(return_annotation, imports)
    } else {
        // No return type annotation - use None (void)
        Ok(quote! {
            ::pyo3_stub_gen::type_info::no_return_type_output
        })
    }
}

/// Convert Python type annotation to TypeInfo token stream
fn type_annotation_to_token_stream(
    expr: &ast::Expr,
    imports: &[String],
) -> Result<TokenStream2> {
    let type_str = expr_to_type_string(expr);

    // Convert imports to token stream
    let import_tokens: Vec<_> = imports.iter().map(|imp| {
        quote! { #imp.into() }
    }).collect();

    Ok(quote! {
        || ::pyo3_stub_gen::TypeInfo {
            name: #type_str.to_string(),
            import: ::std::collections::HashSet::from([#(#import_tokens),*])
        }
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
            format!("{}.{}", expr_to_type_string_inner(&attr.value, false), attr.attr)
        }
        ast::Expr::Subscript(subscript) => {
            let base = expr_to_type_string_inner(&subscript.value, false);
            let slice = expr_to_type_string_inner(&subscript.slice, true);
            format!("{}[{}]", base, slice)
        }
        ast::Expr::List(list) => {
            let elements: Vec<String> = list.elts.iter().map(|e| expr_to_type_string_inner(e, false)).collect();
            format!("[{}]", elements.join(", "))
        }
        ast::Expr::Tuple(tuple) => {
            let elements: Vec<String> = tuple.elts.iter().map(|e| expr_to_type_string_inner(e, in_subscript)).collect();
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
    use quote::quote;

    #[test]
    fn test_basic_function() -> Result<()> {
        let input = quote! {
            r#"
def foo(x: int) -> int:
    """A simple function"""
            "#
        };
        let out = gen_function_from_python_impl(input)?;
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
        let input = quote! {
            r#"
import typing
from collections.abc import Callable

def process(func: Callable[[str], int]) -> typing.Optional[int]:
    """Process a callback function"""
            "#
        };
        let out = gen_function_from_python_impl(input)?;
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
        let input = quote! {
            r#"
import collections.abc
import typing

def fn_override_type(cb: collections.abc.Callable[[str], typing.Any]) -> collections.abc.Callable[[str], typing.Any]:
    """Example function with complex types"""
            "#
        };
        let out = gen_function_from_python_impl(input)?;
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
        let input = quote! {
            r#"
import typing

def add(a: int, b: int, c: typing.Optional[int]) -> int: ...
            "#
        };
        let out = gen_function_from_python_impl(input)?;
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
        let input = quote! {
            r#"
def print_hello(name: str):
    """Print a greeting"""
            "#
        };
        let out = gen_function_from_python_impl(input)?;
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
