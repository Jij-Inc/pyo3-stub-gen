//! Parse Python stub syntax and generate PyFunctionInfo and MethodInfo
//!
//! This module provides functionality to parse Python stub syntax (type hints)
//! and convert them into Rust metadata structures for stub generation.

mod pyfunction;
mod pymethods;

pub use pyfunction::parse_python_function_stub;
pub use pymethods::parse_python_methods_stub;

use indexmap::IndexSet;
use rustpython_parser::ast;
use syn::{Result, Type};

use super::{arg::ArgInfo, attr::DeprecatedInfo, util::TypeOrOverride};

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
