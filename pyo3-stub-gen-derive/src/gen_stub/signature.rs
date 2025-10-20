use std::collections::HashMap;

use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Expr, Ident, Result, Token,
};

use crate::gen_stub::{remove_lifetime, util::TypeOrOverride};

use super::ArgInfo;

#[derive(Debug, Clone, PartialEq)]
enum SignatureArg {
    Ident(Ident),
    Assign(Ident, Token![=], Expr),
    Slash(Token![/]),
    Star(Token![*]),
    Args(Token![*], Ident),
    Keywords(Token![*], Token![*], Ident),
}

impl Parse for SignatureArg {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![/]) {
            Ok(SignatureArg::Slash(input.parse()?))
        } else if input.peek(Token![*]) {
            let star = input.parse()?;
            if input.peek(Token![*]) {
                Ok(SignatureArg::Keywords(star, input.parse()?, input.parse()?))
            } else if input.peek(Ident) {
                Ok(SignatureArg::Args(star, input.parse()?))
            } else {
                Ok(SignatureArg::Star(star))
            }
        } else if input.peek(Ident) {
            let ident = Ident::parse(input)?;
            if input.peek(Token![=]) {
                Ok(SignatureArg::Assign(ident, input.parse()?, input.parse()?))
            } else {
                Ok(SignatureArg::Ident(ident))
            }
        } else {
            dbg!(input);
            todo!()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Signature {
    paren: token::Paren,
    args: Punctuated<SignatureArg, Token![,]>,
}

impl Parse for Signature {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let paren = parenthesized!(content in input);
        let args = content.parse_terminated(SignatureArg::parse, Token![,])?;
        Ok(Self { paren, args })
    }
}

pub struct ArgsWithSignature<'a> {
    pub args: &'a Vec<ArgInfo>,
    pub sig: &'a Option<Signature>,
}

impl ToTokens for ArgsWithSignature<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let param_infos_res: Result<Vec<TokenStream2>> = if let Some(sig) = self.sig {
            // record all Type information from rust's args
            let args_map: HashMap<String, ArgInfo> = self
                .args
                .iter()
                .map(|arg| match arg {
                    ArgInfo {
                        name,
                        r#type: TypeOrOverride::RustType { r#type },
                    } => {
                        let mut ty = r#type.clone();
                        remove_lifetime(&mut ty);
                        (
                            name.clone(),
                            ArgInfo {
                                name: name.clone(),
                                r#type: TypeOrOverride::RustType { r#type: ty },
                            },
                        )
                    }
                    arg @ ArgInfo { name, .. } => (name.clone(), arg.clone()),
                })
                .collect();

            // Track parameter kinds based on position and delimiters
            let mut positional_only = true; // Start in positional-only mode
            let mut after_star = false;

            sig.args.iter().map(|sig_arg| match sig_arg {
                SignatureArg::Slash(_) => {
                    // `/` delimiter - parameters before this are positional-only
                    positional_only = false;
                    Ok(quote! {}) // Don't generate anything for the delimiter itself
                }
                SignatureArg::Star(_) => {
                    // Bare `*` - parameters after this are keyword-only
                    positional_only = false;
                    after_star = true;
                    Ok(quote! {}) // Don't generate anything for the delimiter itself
                }
                SignatureArg::Ident(ident) => {
                    let name = ident.to_string();
                    let kind = if positional_only {
                        quote! { ::pyo3_stub_gen::type_info::ParameterKind::PositionalOnly }
                    } else if after_star {
                        quote! { ::pyo3_stub_gen::type_info::ParameterKind::KeywordOnly }
                    } else {
                        quote! { ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword }
                    };

                    match args_map.get(&name) {
                        Some(ArgInfo { name, r#type: TypeOrOverride::RustType { r#type } }) => Ok(quote! {
                            ::pyo3_stub_gen::type_info::ParameterInfo {
                                name: #name,
                                kind: #kind,
                                type_info: <#r#type as ::pyo3_stub_gen::PyStubType>::type_input,
                                default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                            }
                        }),
                        Some(ArgInfo { name, r#type: TypeOrOverride::OverrideType{ type_repr, imports, .. }}) => {
                            let imports = imports.iter().collect::<Vec<&String>>();
                            Ok(quote! {
                                ::pyo3_stub_gen::type_info::ParameterInfo {
                                    name: #name,
                                    kind: #kind,
                                    type_info: || ::pyo3_stub_gen::TypeInfo {
                                        name: #type_repr.to_string(),
                                        import: ::std::collections::HashSet::from([#(#imports.into(),)*])
                                    },
                                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                                }
                            })
                        },
                        None => Err(syn::Error::new(ident.span(), format!("cannot find argument: {ident}")))
                    }
                }
                SignatureArg::Assign(ident, _eq, value) => {
                    let name = ident.to_string();
                    let kind = if positional_only {
                        quote! { ::pyo3_stub_gen::type_info::ParameterKind::PositionalOnly }
                    } else if after_star {
                        quote! { ::pyo3_stub_gen::type_info::ParameterKind::KeywordOnly }
                    } else {
                        quote! { ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword }
                    };

                    match args_map.get(&name) {
                        Some(ArgInfo { name, r#type: TypeOrOverride::RustType { r#type } }) => {
                            let default = if value.to_token_stream().to_string() == "None" {
                                quote! { "None".to_string() }
                            } else {
                                quote! {
                                    let v: #r#type = #value;
                                    ::pyo3_stub_gen::util::fmt_py_obj(v)
                                }
                            };
                            Ok(quote! {
                                ::pyo3_stub_gen::type_info::ParameterInfo {
                                    name: #name,
                                    kind: #kind,
                                    type_info: <#r#type as ::pyo3_stub_gen::PyStubType>::type_input,
                                    default: ::pyo3_stub_gen::type_info::ParameterDefault::Expr({
                                        fn _fmt() -> String {
                                            #default
                                        }
                                        _fmt
                                    }),
                                }
                            })
                        },
                        Some(ArgInfo { name, r#type: TypeOrOverride::OverrideType{ type_repr, imports, r#type }}) => {
                            let imports = imports.iter().collect::<Vec<&String>>();
                            let default = if value.to_token_stream().to_string() == "None" {
                                quote! { "None".to_string() }
                            } else {
                                quote! {
                                    let v: #r#type = #value;
                                    ::pyo3_stub_gen::util::fmt_py_obj(v)
                                }
                            };
                            Ok(quote! {
                                ::pyo3_stub_gen::type_info::ParameterInfo {
                                    name: #name,
                                    kind: #kind,
                                    type_info: || ::pyo3_stub_gen::TypeInfo {
                                        name: #type_repr.to_string(),
                                        import: ::std::collections::HashSet::from([#(#imports.into(),)*])
                                    },
                                    default: ::pyo3_stub_gen::type_info::ParameterDefault::Expr({
                                        fn _fmt() -> String {
                                            #default
                                        }
                                        _fmt
                                    }),
                                }
                            })
                        },
                        None => Err(syn::Error::new(ident.span(), format!("cannot find argument: {ident}")))
                    }
                },
                SignatureArg::Args(_, ident) => {
                    positional_only = false;
                    after_star = true; // After *args, everything is keyword-only
                    let name = ident.to_string();
                    match args_map.get(&name) {
                        Some(ArgInfo { name, r#type: TypeOrOverride::RustType { r#type } }) => Ok(quote! {
                            ::pyo3_stub_gen::type_info::ParameterInfo {
                                name: #name,
                                kind: ::pyo3_stub_gen::type_info::ParameterKind::VarPositional,
                                type_info: <#r#type as ::pyo3_stub_gen::PyStubType>::type_input,
                                default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                            }
                        }),
                        Some(ArgInfo { name, r#type: TypeOrOverride::OverrideType{ type_repr, imports, .. }}) => {
                            let imports = imports.iter().collect::<Vec<&String>>();
                            Ok(quote! {
                                ::pyo3_stub_gen::type_info::ParameterInfo {
                                    name: #name,
                                    kind: ::pyo3_stub_gen::type_info::ParameterKind::VarPositional,
                                    type_info: || ::pyo3_stub_gen::TypeInfo {
                                        name: #type_repr.to_string(),
                                        import: ::std::collections::HashSet::from([#(#imports.into(),)*])
                                    },
                                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                                }
                            })
                        },
                        None => Err(syn::Error::new(ident.span(), format!("cannot find argument: {ident}")))
                    }
                },
                SignatureArg::Keywords(_, _, ident) => {
                    positional_only = false;
                    let name = ident.to_string();
                    match args_map.get(&name) {
                        Some(ArgInfo { name, r#type: TypeOrOverride::RustType { r#type } }) => Ok(quote! {
                            ::pyo3_stub_gen::type_info::ParameterInfo {
                                name: #name,
                                kind: ::pyo3_stub_gen::type_info::ParameterKind::VarKeyword,
                                type_info: <#r#type as ::pyo3_stub_gen::PyStubType>::type_input,
                                default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                            }
                        }),
                        Some(ArgInfo { name, r#type: TypeOrOverride::OverrideType{ type_repr, imports, .. }}) => {
                            let imports = imports.iter().collect::<Vec<&String>>();
                            Ok(quote! {
                                ::pyo3_stub_gen::type_info::ParameterInfo {
                                    name: #name,
                                    kind: ::pyo3_stub_gen::type_info::ParameterKind::VarKeyword,
                                    type_info: || ::pyo3_stub_gen::TypeInfo {
                                        name: #type_repr.to_string(),
                                        import: ::std::collections::HashSet::from([#(#imports.into(),)*])
                                    },
                                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                                }
                            })
                        },
                        None => Err(syn::Error::new(ident.span(), format!("cannot find argument: {ident}")))
                    }
                }
            }).collect()
        } else {
            // No signature attribute - all parameters are positional or keyword
            self.args
                .iter()
                .map(|arg| {
                    match arg {
                        ArgInfo { name, r#type: TypeOrOverride::RustType { r#type } } => {
                            let mut ty = r#type.clone();
                            remove_lifetime(&mut ty);
                            Ok(quote! {
                                ::pyo3_stub_gen::type_info::ParameterInfo {
                                    name: #name,
                                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                                    type_info: <#ty as ::pyo3_stub_gen::PyStubType>::type_input,
                                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                                }
                            })
                        }
                        ArgInfo { name, r#type: TypeOrOverride::OverrideType{ type_repr, imports, .. }} => {
                            let imports = imports.iter().collect::<Vec<&String>>();
                            Ok(quote! {
                                ::pyo3_stub_gen::type_info::ParameterInfo {
                                    name: #name,
                                    kind: ::pyo3_stub_gen::type_info::ParameterKind::PositionalOrKeyword,
                                    type_info: || ::pyo3_stub_gen::TypeInfo {
                                        name: #type_repr.to_string(),
                                        import: ::std::collections::HashSet::from([#(#imports.into(),)*])
                                    },
                                    default: ::pyo3_stub_gen::type_info::ParameterDefault::None,
                                }
                            })
                        },
                    }
                })
                .collect()
        };
        match param_infos_res {
            Ok(param_infos) => {
                // Filter out empty token streams from delimiters
                let param_infos: Vec<_> = param_infos
                    .into_iter()
                    .filter(|ts| !ts.is_empty())
                    .collect();
                tokens.append_all(quote! { &[ #(#param_infos),* ] })
            },
            Err(err) => tokens.extend(err.to_compile_error()),
        }
    }
}

impl Signature {
    pub fn overriding_operator(sig: &syn::Signature) -> Option<Self> {
        if sig.ident == "__pow__" {
            return Some(syn::parse_str("(exponent, modulo=None)").unwrap());
        }
        if sig.ident == "__rpow__" {
            return Some(syn::parse_str("(base, modulo=None)").unwrap());
        }
        None
    }
}
