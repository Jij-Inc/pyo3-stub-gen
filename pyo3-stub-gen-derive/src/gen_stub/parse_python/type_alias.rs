//! Parse Python type alias stub syntax and generate TypeAliasInfo

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rustpython_parser::{ast, Parse};
use syn::{parse::Parse as SynParse, parse::ParseStream, Error, LitStr, Result};

use super::{dedent, expr_to_type_string};

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
                        type_aliases.push(PythonTypeAliasStub {
                            name: alias_name,
                            type_expr: type_str,
                            imports: imports.clone(),
                        });
                    } else {
                        return Err(Error::new(
                            input.python_stub.span(),
                            format!("Type alias '{}' must have a value", alias_name),
                        ));
                    }
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

            quote! {
                pyo3_stub_gen::inventory::submit! {
                    pyo3_stub_gen::type_info::TypeAliasInfo {
                        name: #name,
                        module: #module,
                        r#type: || pyo3_stub_gen::TypeInfo {
                            name: #type_repr.to_string(),
                            source_module: None,
                            import: {
                                let mut set = std::collections::HashSet::new();
                                #(set.insert(#import_refs);)*
                                set
                            },
                            type_refs: std::collections::HashMap::new(),
                        },
                    }
                }
            }
        })
        .collect();

    Ok(quote! {
        #(#submissions)*
    })
}
