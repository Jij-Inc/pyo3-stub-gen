//! Parse Python stub syntax and generate PyFunctionInfo and MethodInfo
//!
//! This module provides functionality to parse Python stub syntax (type hints)
//! and convert them into Rust metadata structures for stub generation.

mod pyfunction;
mod pymethods;

pub use pyfunction::{
    parse_gen_function_from_python_input, parse_python_function_stub, GenFunctionFromPythonInput,
};
pub use pymethods::parse_python_methods_stub;

use indexmap::IndexSet;
use rustpython_parser::ast;
use syn::{Result, Type};

use super::{
    arg::ArgInfo,
    attr::DeprecatedInfo,
    parameter::DefaultExpr,
    parameter::{ParameterKind, ParameterWithKind, Parameters},
    util::TypeOrOverride,
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

/// Build Parameters directly from Python AST Arguments
///
/// This function constructs Parameters with proper ParameterKind classification
/// based on Python's argument structure (positional-only, keyword-only, varargs, etc.)
pub(super) fn build_parameters_from_ast(
    args: &ast::Arguments,
    imports: &[String],
) -> Result<Parameters> {
    let dummy_type: Type = syn::parse_str("()").unwrap();
    let mut parameters = Vec::new();

    // Helper to process a single argument with default value
    let process_arg_with_default =
        |arg: &ast::ArgWithDefault, kind: ParameterKind| -> Result<Option<ParameterWithKind>> {
            let arg_name = arg.def.arg.to_string();

            // Skip 'self' and 'cls' arguments (they are added automatically in generation)
            if arg_name == "self" || arg_name == "cls" {
                return Ok(None);
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

            let arg_info = ArgInfo {
                name: arg_name,
                r#type: type_override,
            };

            // Convert default value from Python AST to Python string
            let default_expr = if let Some(default) = &arg.default {
                Some(DefaultExpr::Python(python_ast_to_python_string(default)?))
            } else {
                None
            };

            Ok(Some(ParameterWithKind {
                arg_info,
                kind,
                default_expr,
            }))
        };

    // Helper to process vararg or kwarg (ast::Arg, not ast::ArgWithDefault)
    let process_var_arg = |arg: &ast::Arg, kind: ParameterKind| -> Result<ParameterWithKind> {
        let arg_name = arg.arg.to_string();

        let type_override = if let Some(annotation) = &arg.annotation {
            type_annotation_to_type_override(annotation, imports, dummy_type.clone())?
        } else {
            // No type annotation - use Any
            TypeOrOverride::OverrideType {
                r#type: dummy_type.clone(),
                type_repr: "typing.Any".to_string(),
                imports: IndexSet::from(["typing".to_string()]),
            }
        };

        let arg_info = ArgInfo {
            name: arg_name,
            r#type: type_override,
        };

        Ok(ParameterWithKind {
            arg_info,
            kind,
            default_expr: None,
        })
    };

    // Process positional-only arguments (before /)
    for arg in &args.posonlyargs {
        if let Some(param) = process_arg_with_default(arg, ParameterKind::PositionalOnly)? {
            parameters.push(param);
        }
    }

    // Process regular positional/keyword arguments
    for arg in &args.args {
        if let Some(param) = process_arg_with_default(arg, ParameterKind::PositionalOrKeyword)? {
            parameters.push(param);
        }
    }

    // Process *args (vararg)
    if let Some(vararg) = &args.vararg {
        parameters.push(process_var_arg(vararg, ParameterKind::VarPositional)?);
    }

    // Process keyword-only arguments (after *)
    for arg in &args.kwonlyargs {
        if let Some(param) = process_arg_with_default(arg, ParameterKind::KeywordOnly)? {
            parameters.push(param);
        }
    }

    // Process **kwargs (kwarg)
    if let Some(kwarg) = &args.kwarg {
        parameters.push(process_var_arg(kwarg, ParameterKind::VarKeyword)?);
    }

    Ok(Parameters::from_vec(parameters))
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
    // Check for pyo3_stub_gen.RustType["TypeName"] marker
    if let Some(type_name) = extract_rust_type_marker(expr)? {
        let rust_type: Type = syn::parse_str(&type_name).map_err(|e| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to parse Rust type '{}': {}", type_name, e),
            )
        })?;
        return Ok(TypeOrOverride::RustType { r#type: rust_type });
    }

    let type_str = expr_to_type_string(expr)?;

    // Convert imports to IndexSet
    let import_set: IndexSet<String> = imports.iter().map(|s| s.to_string()).collect();

    Ok(TypeOrOverride::OverrideType {
        r#type: dummy_type,
        type_repr: type_str,
        imports: import_set,
    })
}

/// Extract type name from pyo3_stub_gen.RustType["TypeName"]
///
/// Returns Some(type_name) if the expression matches the pattern, None otherwise.
/// Returns an error if the pattern matches but the type name is not a string literal.
fn extract_rust_type_marker(expr: &ast::Expr) -> Result<Option<String>> {
    // Match pattern: pyo3_stub_gen.RustType[...]
    if let ast::Expr::Subscript(subscript) = expr {
        if let ast::Expr::Attribute(attr) = &*subscript.value {
            // Check attribute name is "RustType"
            if attr.attr.as_str() == "RustType" {
                // Check module name is "pyo3_stub_gen"
                if let ast::Expr::Name(name) = &*attr.value {
                    if name.id.as_str() == "pyo3_stub_gen" {
                        // Extract type name from subscript (must be a string literal)
                        if let ast::Expr::Constant(constant) = &*subscript.slice {
                            if let ast::Constant::Str(s) = &constant.value {
                                return Ok(Some(s.to_string()));
                            }
                        }
                        return Err(syn::Error::new(
                            proc_macro2::Span::call_site(),
                            "pyo3_stub_gen.RustType requires a string literal (e.g., RustType[\"MyType\"])",
                        ));
                    }
                }
            }
        }
    }
    Ok(None)
}

/// Convert Python AST expression to Python syntax string
///
/// This converts Python AST expressions like `None`, `True`, `[1, 2]` to Python string representation
/// that can be used directly in stub files.
fn python_ast_to_python_string(expr: &ast::Expr) -> Result<String> {
    match expr {
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
            }
            ast::Constant::Bytes(_) => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Bytes literals are not supported as default values",
            )),
            ast::Constant::Ellipsis => Ok("...".to_string()),
            _ => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Unsupported constant type: {:?}", constant.value),
            )),
        },
        ast::Expr::List(list) => {
            // Recursively convert list elements
            let elements: Result<Vec<_>> =
                list.elts.iter().map(python_ast_to_python_string).collect();
            Ok(format!("[{}]", elements?.join(", ")))
        }
        ast::Expr::Tuple(tuple) => {
            // Recursively convert tuple elements
            let elements: Result<Vec<_>> =
                tuple.elts.iter().map(python_ast_to_python_string).collect();
            let elements = elements?;
            if elements.len() == 1 {
                // Single-element tuple needs trailing comma
                Ok(format!("({},)", elements[0]))
            } else {
                Ok(format!("({})", elements.join(", ")))
            }
        }
        ast::Expr::Dict(dict) => {
            // Recursively convert dict key-value pairs
            let mut pairs = Vec::new();
            for (key_opt, value) in dict.keys.iter().zip(dict.values.iter()) {
                if let Some(key) = key_opt {
                    let key_str = python_ast_to_python_string(key)?;
                    let value_str = python_ast_to_python_string(value)?;
                    pairs.push(format!("{}: {}", key_str, value_str));
                } else {
                    // Handle **kwargs expansion in dict literals
                    return Ok("...".to_string());
                }
            }
            Ok(format!("{{{}}}", pairs.join(", ")))
        }
        ast::Expr::Name(name) => Ok(name.id.to_string()),
        ast::Expr::Attribute(_) => {
            // Handle qualified names like `MyEnum.VARIANT`
            expr_to_type_string(expr)
        }
        ast::Expr::UnaryOp(unary) => {
            // Handle negative numbers
            if matches!(unary.op, ast::UnaryOp::USub) {
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
        _ => {
            // For other expressions, use "..." placeholder
            Ok("...".to_string())
        }
    }
}

/// Convert Python expression to type string
fn expr_to_type_string(expr: &ast::Expr) -> Result<String> {
    expr_to_type_string_inner(expr, false)
}

/// Convert Python expression to type string with context
fn expr_to_type_string_inner(expr: &ast::Expr, in_subscript: bool) -> Result<String> {
    // Check for pyo3_stub_gen.RustType["TypeName"] marker first
    // If found, return just the type name (the marker will be handled elsewhere)
    if let Some(type_name) = extract_rust_type_marker(expr)? {
        return Ok(type_name);
    }

    Ok(match expr {
        ast::Expr::Name(name) => name.id.to_string(),
        ast::Expr::Attribute(attr) => {
            format!(
                "{}.{}",
                expr_to_type_string_inner(&attr.value, false)?,
                attr.attr
            )
        }
        ast::Expr::Subscript(subscript) => {
            let base = expr_to_type_string_inner(&subscript.value, false)?;
            let slice = expr_to_type_string_inner(&subscript.slice, true)?;
            format!("{}[{}]", base, slice)
        }
        ast::Expr::List(list) => {
            let elements: Result<Vec<String>> = list
                .elts
                .iter()
                .map(|e| expr_to_type_string_inner(e, false))
                .collect();
            format!("[{}]", elements?.join(", "))
        }
        ast::Expr::Tuple(tuple) => {
            let elements: Result<Vec<String>> = tuple
                .elts
                .iter()
                .map(|e| expr_to_type_string_inner(e, in_subscript))
                .collect();
            let elements = elements?;
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
    })
}
