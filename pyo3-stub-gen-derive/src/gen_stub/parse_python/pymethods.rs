//! Parse Python class method stub syntax and generate MethodInfo

use indexmap::IndexSet;
use rustpython_parser::{ast, Parse};
use syn::{Error, LitStr, Result, Type};

use super::{
    dedent, extract_deprecated_from_decorators, extract_docstring, extract_return_type,
    type_annotation_to_type_override,
};
use crate::gen_stub::{arg::ArgInfo, method::MethodInfo, method::MethodType, util::TypeOrOverride};

/// Intermediate representation for Python class stub (for methods)
pub struct PythonClassStub {
    pub class_def: ast::StmtClassDef,
    pub imports: Vec<String>,
}

impl TryFrom<PythonClassStub> for (String, Vec<MethodInfo>) {
    type Error = syn::Error;

    fn try_from(stub: PythonClassStub) -> Result<Self> {
        let class_name = stub.class_def.name.to_string();
        let mut methods = Vec::new();

        // Extract methods from class body
        for stmt in &stub.class_def.body {
            match stmt {
                ast::Stmt::FunctionDef(func_def) => {
                    let method = build_method_info(func_def, &stub.imports, false)?;
                    methods.push(method);
                }
                ast::Stmt::AsyncFunctionDef(func_def) => {
                    // Convert AsyncFunctionDef to FunctionDef for uniform processing
                    let sync_func = ast::StmtFunctionDef {
                        range: func_def.range,
                        name: func_def.name.clone(),
                        type_params: func_def.type_params.clone(),
                        args: func_def.args.clone(),
                        body: func_def.body.clone(),
                        decorator_list: func_def.decorator_list.clone(),
                        returns: func_def.returns.clone(),
                        type_comment: func_def.type_comment.clone(),
                    };
                    let method = build_method_info(&sync_func, &stub.imports, true)?;
                    methods.push(method);
                }
                _ => {
                    // Ignore other statements (e.g., docstrings, pass)
                }
            }
        }

        if methods.is_empty() {
            return Err(Error::new(
                proc_macro2::Span::call_site(),
                "No method definitions found in class body",
            ));
        }

        Ok((class_name, methods))
    }
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

    // Generate methods using TryFrom
    let stub = PythonClassStub { class_def, imports };
    <(String, Vec<MethodInfo>)>::try_from(stub)
        .map_err(|e| Error::new(input.span(), format!("{}", e)))
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
        if idx == 0
            && ((method_type == MethodType::Instance && arg_name == "self")
                || (method_type == MethodType::Class && arg_name == "cls")
                || (method_type == MethodType::New && arg_name == "cls"))
        {
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

#[cfg(test)]
mod test {
    use super::*;
    use proc_macro2::TokenStream as TokenStream2;
    use quote::{quote, ToTokens};

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
