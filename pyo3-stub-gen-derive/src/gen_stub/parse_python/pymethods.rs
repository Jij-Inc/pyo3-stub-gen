//! Parse Python class method stub syntax and generate MethodInfo

use rustpython_parser::{ast, Parse};
use syn::{Error, LitStr, Result, Type};

use super::pyfunction::PythonFunctionStub;
use super::{
    build_parameters_from_ast, dedent, extract_deprecated_from_decorators, extract_docstring,
    extract_return_type,
};
use crate::gen_stub::{method::MethodInfo, method::MethodType, pymethods::PyMethodsInfo};

/// Intermediate representation for Python method stub
pub struct PythonMethodStub {
    pub func_stub: PythonFunctionStub,
    pub method_type: MethodType,
}

impl TryFrom<PythonMethodStub> for MethodInfo {
    type Error = syn::Error;

    fn try_from(stub: PythonMethodStub) -> Result<Self> {
        let func_name = stub.func_stub.func_def.name.to_string();

        // Extract docstring
        let doc = extract_docstring(&stub.func_stub.func_def);

        // Build Parameters directly from Python AST with proper kind classification
        let parameters =
            build_parameters_from_ast(&stub.func_stub.func_def.args, &stub.func_stub.imports)?;

        // For instance/class/new methods, the first parameter (self/cls) is handled by Python's AST
        // but we need to skip it in our parameters list since it's implied by the method type
        // (The build_parameters_from_ast already skips 'self', so no additional filtering needed)

        // Extract return type
        let return_type =
            extract_return_type(&stub.func_stub.func_def.returns, &stub.func_stub.imports)?;

        // Try to extract deprecated decorator
        let deprecated =
            extract_deprecated_from_decorators(&stub.func_stub.func_def.decorator_list);

        // Construct MethodInfo
        Ok(MethodInfo {
            name: func_name,
            parameters,
            r#return: return_type,
            doc,
            r#type: stub.method_type,
            is_async: stub.func_stub.is_async,
            deprecated,
            type_ignored: None,
        })
    }
}

/// Intermediate representation for Python class stub (for methods)
pub struct PythonClassStub {
    pub class_def: ast::StmtClassDef,
    pub imports: Vec<String>,
}

impl PythonClassStub {
    /// Parse Python class definition from a literal string
    pub fn new(input: &LitStr) -> Result<Self> {
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

        Ok(Self { class_def, imports })
    }
}

impl TryFrom<PythonClassStub> for PyMethodsInfo {
    type Error = syn::Error;

    fn try_from(stub: PythonClassStub) -> Result<Self> {
        let class_name = stub.class_def.name.to_string();
        let mut methods = Vec::new();

        // Extract methods from class body
        for stmt in &stub.class_def.body {
            match stmt {
                ast::Stmt::FunctionDef(func_def) => {
                    // Determine method type
                    let method_type = determine_method_type(func_def, &func_def.args);

                    // Create PythonFunctionStub
                    let func_stub = PythonFunctionStub {
                        func_def: func_def.clone(),
                        imports: stub.imports.clone(),
                        is_async: false,
                    };

                    // Create PythonMethodStub and convert to MethodInfo
                    let method_stub = PythonMethodStub {
                        func_stub,
                        method_type,
                    };
                    let method = MethodInfo::try_from(method_stub)?;
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

                    // Determine method type
                    let method_type = determine_method_type(&sync_func, &sync_func.args);

                    // Create PythonFunctionStub
                    let func_stub = PythonFunctionStub {
                        func_def: sync_func,
                        imports: stub.imports.clone(),
                        is_async: true,
                    };

                    // Create PythonMethodStub and convert to MethodInfo
                    let method_stub = PythonMethodStub {
                        func_stub,
                        method_type,
                    };
                    let method = MethodInfo::try_from(method_stub)?;
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

        // Parse class name as Type
        let struct_id: Type = syn::parse_str(&class_name).map_err(|e| {
            Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to parse class name '{}': {}", class_name, e),
            )
        })?;

        Ok(PyMethodsInfo {
            struct_id,
            attrs: Vec::new(),
            getters: Vec::new(),
            setters: Vec::new(),
            methods,
        })
    }
}

/// Parse Python class definition and return PyMethodsInfo
pub fn parse_python_methods_stub(input: &LitStr) -> Result<PyMethodsInfo> {
    let stub = PythonClassStub::new(input)?;
    PyMethodsInfo::try_from(stub).map_err(|e| Error::new(input.span(), format!("{}", e)))
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
        let py_methods_info = parse_python_methods_stub(&stub_str)?;
        assert_eq!(py_methods_info.methods.len(), 1);

        let out = py_methods_info.methods[0].to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::MethodInfo {
            name: "increment",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "x",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "int".to_string(),
                        import: ::std::collections::HashSet::from([]),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
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
        let py_methods_info = parse_python_methods_stub(&stub_str)?;
        assert_eq!(py_methods_info.methods.len(), 2);

        assert_eq!(py_methods_info.methods[0].name, "increment_1");
        assert_eq!(py_methods_info.methods[1].name, "increment_2");
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
        let py_methods_info = parse_python_methods_stub(&stub_str)?;
        assert_eq!(py_methods_info.methods.len(), 1);

        let out = py_methods_info.methods[0].to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::MethodInfo {
            name: "create",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "name",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "str".to_string(),
                        import: ::std::collections::HashSet::from([]),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
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
        let py_methods_info = parse_python_methods_stub(&stub_str)?;
        assert_eq!(py_methods_info.methods.len(), 1);

        let out = py_methods_info.methods[0].to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::MethodInfo {
            name: "from_string",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "s",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "str".to_string(),
                        import: ::std::collections::HashSet::from([]),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
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
        let py_methods_info = parse_python_methods_stub(&stub_str)?;
        assert_eq!(py_methods_info.methods.len(), 1);

        let out = py_methods_info.methods[0].to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::MethodInfo {
            name: "__new__",
            parameters: &[],
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
        let py_methods_info = parse_python_methods_stub(&stub_str)?;
        assert_eq!(py_methods_info.methods.len(), 1);

        let out = py_methods_info.methods[0].to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::MethodInfo {
            name: "process",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "func",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "Callable[[str], int]".to_string(),
                        import: ::std::collections::HashSet::from([
                            "typing".into(),
                            "collections.abc".into(),
                        ]),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
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
        let py_methods_info = parse_python_methods_stub(&stub_str)?;
        assert_eq!(py_methods_info.methods.len(), 1);

        let out = py_methods_info.methods[0].to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::MethodInfo {
            name: "fetch_data",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "url",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: || ::pyo3_stub_gen::TypeInfo {
                        name: "str".to_string(),
                        import: ::std::collections::HashSet::from([]),
                    },
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
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

    #[test]
    fn test_rust_type_marker_in_method() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            class PyProblem:
                def __iadd__(self, other: pyo3_stub_gen.RustType["SomeRustType"]) -> pyo3_stub_gen.RustType["PyProblem"]:
                    """In-place addition using Rust type marker"""
            "#
        })?;
        let py_methods_info = parse_python_methods_stub(&stub_str)?;
        assert_eq!(py_methods_info.methods.len(), 1);

        let out = py_methods_info.methods[0].to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::MethodInfo {
            name: "__iadd__",
            parameters: &[
                ::pyo3_stub_gen::type_info::ParameterInfo {
                    name: "other",
                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                    type_info: <SomeRustType as ::pyo3_stub_gen::PyStubType>::type_input,
                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                },
            ],
            r#return: <PyProblem as pyo3_stub_gen::PyStubType>::type_output,
            doc: "In-place addition using Rust type marker",
            r#type: ::pyo3_stub_gen::type_info::MethodType::Instance,
            is_async: false,
            deprecated: None,
            type_ignored: None,
        }
        "###);
        Ok(())
    }

    #[test]
    fn test_keyword_only_params_with_defaults() -> Result<()> {
        let stub_str: LitStr = syn::parse2(quote! {
            r#"
            import builtins
            import typing

            class Placeholder:
                def configure(
                    self,
                    name: builtins.str,
                    *,
                    dtype: builtins.str,
                    ndim: builtins.int,
                    shape: typing.Optional[builtins.str],
                    jagged: builtins.bool = False,
                    latex: typing.Optional[builtins.str] = None,
                    description: typing.Optional[builtins.str] = None,
                ) -> pyo3_stub_gen.RustType["Placeholder"]:
                    """
                    Configure placeholder with keyword-only parameters.

                    This demonstrates keyword-only parameters (after *) which should be
                    preserved in the generated stub file.
                    """
            "#
        })?;
        let py_methods_info = parse_python_methods_stub(&stub_str)?;
        assert_eq!(py_methods_info.methods.len(), 1);

        let out = py_methods_info.to_token_stream();
        insta::assert_snapshot!(format_as_value(out), @r###"
        ::pyo3_stub_gen::type_info::PyMethodsInfo {
            struct_id: std::any::TypeId::of::<Placeholder>,
            attrs: &[],
            getters: &[],
            setters: &[],
            methods: &[
                ::pyo3_stub_gen::type_info::MethodInfo {
                    name: "configure",
                    parameters: &[
                        ::pyo3_stub_gen::type_info::ParameterInfo {
                            name: "name",
                            kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                            type_info: || ::pyo3_stub_gen::TypeInfo {
                                name: "builtins.str".to_string(),
                                import: ::std::collections::HashSet::from([
                                    "builtins".into(),
                                    "typing".into(),
                                ]),
                            },
                            default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                        },
                        ::pyo3_stub_gen::type_info::ParameterInfo {
                            name: "dtype",
                            kind: ::pyo3_stub_gen::type_info::ParameterKind::KeywordOnly,
                            type_info: || ::pyo3_stub_gen::TypeInfo {
                                name: "builtins.str".to_string(),
                                import: ::std::collections::HashSet::from([
                                    "builtins".into(),
                                    "typing".into(),
                                ]),
                            },
                            default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                        },
                        ::pyo3_stub_gen::type_info::ParameterInfo {
                            name: "ndim",
                            kind: ::pyo3_stub_gen::type_info::ParameterKind::KeywordOnly,
                            type_info: || ::pyo3_stub_gen::TypeInfo {
                                name: "builtins.int".to_string(),
                                import: ::std::collections::HashSet::from([
                                    "builtins".into(),
                                    "typing".into(),
                                ]),
                            },
                            default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                        },
                        ::pyo3_stub_gen::type_info::ParameterInfo {
                            name: "shape",
                            kind: ::pyo3_stub_gen::type_info::ParameterKind::KeywordOnly,
                            type_info: || ::pyo3_stub_gen::TypeInfo {
                                name: "typing.Optional[builtins.str]".to_string(),
                                import: ::std::collections::HashSet::from([
                                    "builtins".into(),
                                    "typing".into(),
                                ]),
                            },
                            default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                        },
                        ::pyo3_stub_gen::type_info::ParameterInfo {
                            name: "jagged",
                            kind: ::pyo3_stub_gen::type_info::ParameterKind::KeywordOnly,
                            type_info: || ::pyo3_stub_gen::TypeInfo {
                                name: "builtins.bool".to_string(),
                                import: ::std::collections::HashSet::from([
                                    "builtins".into(),
                                    "typing".into(),
                                ]),
                            },
                            default: ::pyo3_stub_gen::type_info::ParameterDefault::Expr({
                                fn _fmt() -> String {
                                    "False".to_string()
                                }
                                _fmt
                            }),
                        },
                        ::pyo3_stub_gen::type_info::ParameterInfo {
                            name: "latex",
                            kind: ::pyo3_stub_gen::type_info::ParameterKind::KeywordOnly,
                            type_info: || ::pyo3_stub_gen::TypeInfo {
                                name: "typing.Optional[builtins.str]".to_string(),
                                import: ::std::collections::HashSet::from([
                                    "builtins".into(),
                                    "typing".into(),
                                ]),
                            },
                            default: ::pyo3_stub_gen::type_info::ParameterDefault::Expr({
                                fn _fmt() -> String {
                                    "None".to_string()
                                }
                                _fmt
                            }),
                        },
                        ::pyo3_stub_gen::type_info::ParameterInfo {
                            name: "description",
                            kind: ::pyo3_stub_gen::type_info::ParameterKind::KeywordOnly,
                            type_info: || ::pyo3_stub_gen::TypeInfo {
                                name: "typing.Optional[builtins.str]".to_string(),
                                import: ::std::collections::HashSet::from([
                                    "builtins".into(),
                                    "typing".into(),
                                ]),
                            },
                            default: ::pyo3_stub_gen::type_info::ParameterDefault::Expr({
                                fn _fmt() -> String {
                                    "None".to_string()
                                }
                                _fmt
                            }),
                        },
                    ],
                    r#return: <Placeholder as pyo3_stub_gen::PyStubType>::type_output,
                    doc: "\n        Configure placeholder with keyword-only parameters.\n\n        This demonstrates keyword-only parameters (after *) which should be\n        preserved in the generated stub file.\n        ",
                    r#type: ::pyo3_stub_gen::type_info::MethodType::Instance,
                    is_async: false,
                    deprecated: None,
                    type_ignored: None,
                },
            ],
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
