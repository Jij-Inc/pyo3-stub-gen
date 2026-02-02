//! Parse Python type alias stub syntax and generate TypeAliasInfo

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rustpython_parser::{ast, Parse};
use syn::{parse::Parse as SynParse, parse::ParseStream, Error, LitStr, Result};

use super::{collect_rust_type_markers, dedent, expr_to_type_string};

/// Input for gen_type_alias_from_python! macro
pub struct GenTypeAliasFromPythonInput {
    pub module: String,
    pub python_stub: LitStr,
}

impl SynParse for GenTypeAliasFromPythonInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // First parameter is module name (string literal)
        let module: LitStr = input.parse()?;
        let _: syn::token::Comma = input.parse()?;

        // Second parameter is Python stub code
        let python_stub: LitStr = input.parse()?;

        Ok(Self {
            module: module.value(),
            python_stub,
        })
    }
}

/// Intermediate representation for Python type alias stub
pub struct PythonTypeAliasStub {
    pub name: String,
    pub type_expr: String,
    pub imports: Vec<String>,
    pub rust_type_markers: Vec<String>,
    pub doc: String,
}

/// Extract next-line docstring for type alias (Pyright's convention)
fn extract_type_alias_docstring(stmts: &[ast::Stmt], current_index: usize) -> String {
    // Check if next statement is a string literal
    if let Some(ast::Stmt::Expr(expr_stmt)) = stmts.get(current_index + 1) {
        if let ast::Expr::Constant(constant) = &*expr_stmt.value {
            if let ast::Constant::Str(s) = &constant.value {
                return s.to_string();
            }
        }
    }
    String::new()
}

/// Parse Python type alias stub string and return Vec<TypeAliasInfo> as TokenStream
pub fn parse_python_type_alias_stub(input: &GenTypeAliasFromPythonInput) -> Result<TokenStream2> {
    let stub_content = input.python_stub.value();

    // Remove common indentation
    let dedented_content = dedent(&stub_content);

    // Parse Python code
    let parsed = ast::Suite::parse(&dedented_content, "<stub>").map_err(|e| {
        Error::new(
            input.python_stub.span(),
            format!("Failed to parse Python stub: {}", e),
        )
    })?;

    // Extract imports and type alias definitions
    let mut imports = Vec::new();
    let mut type_aliases = Vec::new();

    for (idx, stmt) in parsed.iter().enumerate() {
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
            ast::Stmt::AnnAssign(ann_assign) => {
                // Type alias pattern: Name: TypeAlias = Type
                if let ast::Expr::Name(name_expr) = &*ann_assign.target {
                    let alias_name = name_expr.id.to_string();

                    // Check if annotation is TypeAlias
                    let is_type_alias = match &*ann_assign.annotation {
                        ast::Expr::Name(ann_name) => ann_name.id.as_str() == "TypeAlias",
                        ast::Expr::Attribute(attr) => attr.attr.as_str() == "TypeAlias",
                        _ => false,
                    };

                    if !is_type_alias {
                        return Err(Error::new(
                            input.python_stub.span(),
                            format!("Type alias '{}' must have TypeAlias annotation", alias_name),
                        ));
                    }

                    // Extract the actual type from the value
                    if let Some(value) = &ann_assign.value {
                        let type_str = expr_to_type_string(value)?;
                        let rust_type_markers = collect_rust_type_markers(value)?;
                        let doc = extract_type_alias_docstring(&parsed, idx);
                        type_aliases.push(PythonTypeAliasStub {
                            name: alias_name,
                            type_expr: type_str,
                            imports: imports.clone(),
                            rust_type_markers,
                            doc,
                        });
                    } else {
                        return Err(Error::new(
                            input.python_stub.span(),
                            format!("Type alias '{}' must have a value", alias_name),
                        ));
                    }
                }
            }
            ast::Stmt::TypeAlias(type_alias_stmt) => {
                // Python 3.12+ type statement: type Name = Type
                if let ast::Expr::Name(name_expr) = &*type_alias_stmt.name {
                    let alias_name = name_expr.id.to_string();
                    let type_str = expr_to_type_string(&type_alias_stmt.value)?;
                    let rust_type_markers = collect_rust_type_markers(&type_alias_stmt.value)?;
                    let doc = extract_type_alias_docstring(&parsed, idx);
                    type_aliases.push(PythonTypeAliasStub {
                        name: alias_name,
                        type_expr: type_str,
                        imports: imports.clone(),
                        rust_type_markers,
                        doc,
                    });
                }
            }
            _ => {
                // Ignore other statements
            }
        }
    }

    if type_aliases.is_empty() {
        return Err(Error::new(
            input.python_stub.span(),
            "No type alias definitions found in Python stub",
        ));
    }

    // Generate TypeAliasInfo submissions for each alias
    let module = &input.module;
    let submissions: Vec<TokenStream2> = type_aliases
        .into_iter()
        .map(|alias| {
            let name = alias.name;
            let type_repr = alias.type_expr;
            let doc = alias.doc;
            let has_rust_markers = !alias.rust_type_markers.is_empty();

            let import_refs: Vec<TokenStream2> = alias
                .imports
                .iter()
                .map(|imp| {
                    quote! {
                        pyo3_stub_gen::ImportRef::Module(
                            pyo3_stub_gen::ModuleRef::Named(#imp.to_string())
                        )
                    }
                })
                .collect();

            // Generate code for processing RustType markers at runtime
            let (type_name_code, type_refs_code) = if has_rust_markers {
                // Parse rust_type_markers as syn::Type
                let marker_types: Vec<syn::Type> = alias
                    .rust_type_markers
                    .iter()
                    .filter_map(|s| syn::parse_str(s).ok())
                    .collect();

                let rust_names = alias.rust_type_markers.iter().collect::<Vec<_>>();

                (
                    quote! {
                        {
                            let mut type_name = #type_repr.to_string();
                            #(
                                let type_info = <#marker_types as ::pyo3_stub_gen::PyStubType>::type_input();
                                type_name = type_name.replace(#rust_names, &type_info.name);
                            )*
                            type_name
                        }
                    },
                    quote! {
                        {
                            let mut type_refs = ::std::collections::HashMap::new();
                            #(
                                let type_info = <#marker_types as ::pyo3_stub_gen::PyStubType>::type_input();
                                if let Some(module) = type_info.source_module {
                                    type_refs.insert(
                                        type_info.name.split('[').next().unwrap_or(&type_info.name)
                                            .split('.').last().unwrap_or(&type_info.name).to_string(),
                                        ::pyo3_stub_gen::TypeIdentifierRef {
                                            module: module.into(),
                                            import_kind: ::pyo3_stub_gen::ImportKind::Module,
                                        }
                                    );
                                }
                                type_refs.extend(type_info.type_refs);
                            )*
                            type_refs
                        }
                    },
                )
            } else {
                // No RustType markers - use simple static values
                (
                    quote! { #type_repr.to_string() },
                    quote! { ::std::collections::HashMap::new() },
                )
            };

            quote! {
                pyo3_stub_gen::inventory::submit! {
                    pyo3_stub_gen::type_info::TypeAliasInfo {
                        name: #name,
                        module: #module,
                        r#type: || pyo3_stub_gen::TypeInfo {
                            name: #type_name_code,
                            source_module: None,
                            import: {
                                let mut set = std::collections::HashSet::new();
                                #(set.insert(#import_refs);)*
                                set
                            },
                            type_refs: #type_refs_code,
                        },
                        doc: #doc,
                        file: file!(),
                        line: line!(),
                        column: column!(),
                    }
                }
            }
        })
        .collect();

    Ok(quote! {
        #(#submissions)*
    })
}
