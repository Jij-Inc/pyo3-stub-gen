//! Parameter intermediate representation for derive macros
//!
//! This module provides intermediate representations for parameters that are used
//! during the code generation phase. These types exist only within the derive macro
//! and are converted to `::pyo3_stub_gen::type_info::ParameterInfo` via `ToTokens`.

use std::collections::HashMap;

use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Expr, Result};

use super::{remove_lifetime, signature::SignatureArg, util::TypeOrOverride, ArgInfo, Signature};

/// Intermediate representation for a parameter with its kind determined
#[derive(Debug, Clone)]
pub(crate) struct ParameterWithKind {
    pub(crate) arg_info: ArgInfo,
    pub(crate) kind: ParameterKindIntermediate,
    pub(crate) default_expr: Option<Expr>,
}

impl ToTokens for ParameterWithKind {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.arg_info.name;
        let kind = &self.kind;

        let default_tokens = if let Some(value) = &self.default_expr {
            match &self.arg_info.r#type {
                TypeOrOverride::RustType { r#type } => {
                    let default = if value.to_token_stream().to_string() == "None" {
                        quote! { "None".to_string() }
                    } else {
                        quote! {
                            let v: #r#type = #value;
                            ::pyo3_stub_gen::util::fmt_py_obj(v)
                        }
                    };
                    quote! {
                        ::pyo3_stub_gen::type_info::ParameterDefault::Expr({
                            fn _fmt() -> String {
                                #default
                            }
                            _fmt
                        })
                    }
                }
                TypeOrOverride::OverrideType { .. } => {
                    // For OverrideType, convert the default value expression directly to a string
                    // since r#type may be a dummy type and we can't use it for type annotations
                    let mut value_str = value.to_token_stream().to_string();
                    // Convert Rust bool literals to Python bool literals
                    if value_str == "false" {
                        value_str = "False".to_string();
                    } else if value_str == "true" {
                        value_str = "True".to_string();
                    }
                    quote! {
                        ::pyo3_stub_gen::type_info::ParameterDefault::Expr({
                            fn _fmt() -> String {
                                #value_str.to_string()
                            }
                            _fmt
                        })
                    }
                }
            }
        } else {
            quote! { ::pyo3_stub_gen::type_info::ParameterDefault::None }
        };

        let param_info = match &self.arg_info.r#type {
            TypeOrOverride::RustType { r#type } => {
                quote! {
                    ::pyo3_stub_gen::type_info::ParameterInfo {
                        name: #name,
                        kind: #kind,
                        type_info: <#r#type as ::pyo3_stub_gen::PyStubType>::type_input,
                        default: #default_tokens,
                    }
                }
            }
            TypeOrOverride::OverrideType {
                type_repr, imports, ..
            } => {
                let imports = imports.iter().collect::<Vec<&String>>();
                quote! {
                    ::pyo3_stub_gen::type_info::ParameterInfo {
                        name: #name,
                        kind: #kind,
                        type_info: || ::pyo3_stub_gen::TypeInfo {
                            name: #type_repr.to_string(),
                            import: ::std::collections::HashSet::from([#(#imports.into(),)*])
                        },
                        default: #default_tokens,
                    }
                }
            }
        };

        tokens.append_all(param_info);
    }
}

/// Parameter kind for intermediate representation in derive macro
///
/// This enum mirrors `::pyo3_stub_gen::type_info::ParameterKind` but exists
/// in the derive macro context for code generation purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParameterKindIntermediate {
    PositionalOnly,
    PositionalOrKeyword,
    KeywordOnly,
    VarPositional,
    VarKeyword,
}

impl ToTokens for ParameterKindIntermediate {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let kind_tokens = match self {
            Self::PositionalOnly => {
                quote! { ::pyo3_stub_gen::type_info::ParameterKind::PositionalOnly }
            }
            Self::PositionalOrKeyword => {
                quote! { ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword }
            }
            Self::KeywordOnly => {
                quote! { ::pyo3_stub_gen::type_info::ParameterKind::KeywordOnly }
            }
            Self::VarPositional => {
                quote! { ::pyo3_stub_gen::type_info::ParameterKind::VarPositional }
            }
            Self::VarKeyword => {
                quote! { ::pyo3_stub_gen::type_info::ParameterKind::VarKeyword }
            }
        };
        tokens.append_all(kind_tokens);
    }
}

/// Collection of parameters with their kinds determined
///
/// This newtype wraps `Vec<ParameterWithKind>` and provides constructors that
/// parse PyO3 signature attributes and classify parameters accordingly.
#[derive(Debug, Clone)]
pub(crate) struct Parameters(Vec<ParameterWithKind>);

impl Parameters {
    /// Create Parameters from a Vec<ParameterWithKind>
    ///
    /// This is used when parameters are already classified (e.g., from Python AST).
    pub(crate) fn from_vec(parameters: Vec<ParameterWithKind>) -> Self {
        Self(parameters)
    }

    /// Get mutable access to internal parameters
    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut ParameterWithKind> {
        self.0.iter_mut()
    }

    /// Create parameters without signature attribute
    ///
    /// All parameters will be classified as `PositionalOrKeyword`.
    pub(crate) fn new(args: &[ArgInfo]) -> Self {
        let parameters = args
            .iter()
            .map(|arg| {
                let mut arg_with_clean_type = arg.clone();
                if let ArgInfo {
                    r#type: TypeOrOverride::RustType { r#type },
                    ..
                } = &mut arg_with_clean_type
                {
                    remove_lifetime(r#type);
                }
                ParameterWithKind {
                    arg_info: arg_with_clean_type,
                    kind: ParameterKindIntermediate::PositionalOrKeyword,
                    default_expr: None,
                }
            })
            .collect();
        Self(parameters)
    }

    /// Create parameters with signature attribute
    ///
    /// Parses the signature to determine parameter kinds based on delimiters
    /// (`/` for positional-only, `*` for keyword-only, etc.).
    pub(crate) fn new_with_sig(args: &[ArgInfo], sig: &Signature) -> Result<Self> {
        // Build a map of argument names to their type information
        let args_map: HashMap<String, ArgInfo> = args
            .iter()
            .map(|arg| {
                let mut arg_with_clean_type = arg.clone();
                if let ArgInfo {
                    r#type: TypeOrOverride::RustType { r#type },
                    ..
                } = &mut arg_with_clean_type
                {
                    remove_lifetime(r#type);
                }
                (arg.name.clone(), arg_with_clean_type)
            })
            .collect();

        // Track parameter kinds based on position and delimiters
        let mut positional_only = true; // Start in positional-only mode
        let mut after_star = false;
        let mut parameters = Vec::new();

        for sig_arg in sig.args() {
            match sig_arg {
                SignatureArg::Slash(_) => {
                    // `/` delimiter - parameters before this are positional-only
                    positional_only = false;
                }
                SignatureArg::Star(_) => {
                    // Bare `*` - parameters after this are keyword-only
                    positional_only = false;
                    after_star = true;
                }
                SignatureArg::Ident(ident) => {
                    let name = ident.to_string();
                    let kind = if positional_only {
                        ParameterKindIntermediate::PositionalOnly
                    } else if after_star {
                        ParameterKindIntermediate::KeywordOnly
                    } else {
                        ParameterKindIntermediate::PositionalOrKeyword
                    };

                    let arg_info = args_map
                        .get(&name)
                        .ok_or_else(|| {
                            syn::Error::new(ident.span(), format!("cannot find argument: {}", name))
                        })?
                        .clone();

                    parameters.push(ParameterWithKind {
                        arg_info,
                        kind,
                        default_expr: None,
                    });
                }
                SignatureArg::Assign(ident, _eq, value) => {
                    let name = ident.to_string();
                    let kind = if positional_only {
                        ParameterKindIntermediate::PositionalOnly
                    } else if after_star {
                        ParameterKindIntermediate::KeywordOnly
                    } else {
                        ParameterKindIntermediate::PositionalOrKeyword
                    };

                    let arg_info = args_map
                        .get(&name)
                        .ok_or_else(|| {
                            syn::Error::new(ident.span(), format!("cannot find argument: {}", name))
                        })?
                        .clone();

                    parameters.push(ParameterWithKind {
                        arg_info,
                        kind,
                        default_expr: Some(value.clone()),
                    });
                }
                SignatureArg::Args(_, ident) => {
                    positional_only = false;
                    after_star = true; // After *args, everything is keyword-only
                    let name = ident.to_string();

                    let arg_info = args_map
                        .get(&name)
                        .ok_or_else(|| {
                            syn::Error::new(ident.span(), format!("cannot find argument: {}", name))
                        })?
                        .clone();

                    parameters.push(ParameterWithKind {
                        arg_info,
                        kind: ParameterKindIntermediate::VarPositional,
                        default_expr: None,
                    });
                }
                SignatureArg::Keywords(_, _, ident) => {
                    positional_only = false;
                    let name = ident.to_string();

                    let arg_info = args_map
                        .get(&name)
                        .ok_or_else(|| {
                            syn::Error::new(ident.span(), format!("cannot find argument: {}", name))
                        })?
                        .clone();

                    parameters.push(ParameterWithKind {
                        arg_info,
                        kind: ParameterKindIntermediate::VarKeyword,
                        default_expr: None,
                    });
                }
            }
        }

        Ok(Self(parameters))
    }
}

impl ToTokens for Parameters {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let params = &self.0;
        tokens.append_all(quote! { &[ #(#params),* ] })
    }
}
