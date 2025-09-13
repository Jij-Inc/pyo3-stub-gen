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
    Star(Token![*]),
    Args(Token![*], Ident),
    Keywords(Token![*], Token![*], Ident),
}

impl Parse for SignatureArg {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![*]) {
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
        let arg_infos_res: Result<Vec<TokenStream2>> = if let Some(sig) = self.sig {
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
            sig.args.iter().map(|sig_arg| match sig_arg {
                SignatureArg::Ident(ident) => {
                    let name = ident.to_string();
                    match args_map.get(&name) {
                        Some(ArgInfo { name, r#type: TypeOrOverride::RustType { r#type } }) => Ok(quote! {
                        ::pyo3_stub_gen::type_info::ArgInfo {
                            name: #name,
                            r#type: <#r#type as ::pyo3_stub_gen::PyStubType>::type_input,
                            signature: Some(pyo3_stub_gen::type_info::SignatureArg::Ident),
                        }}),
                        Some(ArgInfo { name, r#type: TypeOrOverride::OverrideType{ type_repr, imports, .. }}) => {
                            let imports = imports.iter().collect::<Vec<&String>>();
                            Ok(quote! {
                            ::pyo3_stub_gen::type_info::ArgInfo {
                                name: #name,
                                r#type: || ::pyo3_stub_gen::TypeInfo { name: #type_repr.to_string(), import: ::std::collections::HashSet::from([#(#imports.into(),)*]) },
                                signature: Some(pyo3_stub_gen::type_info::SignatureArg::Ident),
                            }})
                        },
                        None => Err(syn::Error::new(ident.span(), format!("can not find argument: {ident}")))
                    }
                }
                SignatureArg::Assign(ident, _eq, value) => {
                    let name = ident.to_string();

                    match args_map.get(&name) {
                        Some(ArgInfo { name, r#type: TypeOrOverride::RustType { r#type } }) => {
                            let default = if value.to_token_stream().to_string() == "None" {
                                quote! {
                                "None".to_string()
                                }
                            } else {
                                quote! {
                                let v: #r#type = #value;
                                ::pyo3_stub_gen::util::fmt_py_obj(v)
                                }
                            };
                            Ok(quote! {
                            ::pyo3_stub_gen::type_info::ArgInfo {
                                name: #name,
                                r#type: <#r#type as ::pyo3_stub_gen::PyStubType>::type_input,
                                signature: Some(pyo3_stub_gen::type_info::SignatureArg::Assign{
                                    default: {
                                        fn _fmt() -> String {
                                            #default
                                        }
                                        _fmt
                                    }
                                }),
                            }})
                        },
                        Some(ArgInfo { name, r#type: TypeOrOverride::OverrideType{ type_repr, imports, r#type }}) => {
                            let imports = imports.iter().collect::<Vec<&String>>();
                            let default = if value.to_token_stream().to_string() == "None" {
                                quote! {
                                "None".to_string()
                                }
                            } else {
                                quote! {
                                let v: #r#type = #value;
                                ::pyo3_stub_gen::util::fmt_py_obj(v)
                                }
                            };
                            Ok(quote! {
                            ::pyo3_stub_gen::type_info::ArgInfo {
                                name: #name,
                                r#type: || ::pyo3_stub_gen::TypeInfo { name: #type_repr.to_string(), import: ::std::collections::HashSet::from([#(#imports.into(),)*]) },
                                signature: Some(pyo3_stub_gen::type_info::SignatureArg::Assign{
                                    default: {
                                        fn _fmt() -> String {
                                            #default
                                        }
                                        _fmt
                                    }
                                }),
                            }})
                        },
                        None => Err(syn::Error::new(ident.span(), format!("can not find argument: {ident}")))
                    }
                },
                SignatureArg::Star(_) =>Ok(quote! {
                    ::pyo3_stub_gen::type_info::ArgInfo {
                        name: "",
                        r#type: <() as ::pyo3_stub_gen::PyStubType>::type_input,
                        signature: Some(pyo3_stub_gen::type_info::SignatureArg::Star),
                }}),
                SignatureArg::Args(_, ident) => {
                    let name = ident.to_string();
                    match args_map.get(&name) {
                        Some(ArgInfo { name, r#type: TypeOrOverride::RustType { r#type } }) => Ok(quote! {
                        ::pyo3_stub_gen::type_info::ArgInfo {
                            name: #name,
                            r#type: <#r#type as ::pyo3_stub_gen::PyStubType>::type_input,
                            signature: Some(pyo3_stub_gen::type_info::SignatureArg::Args),
                        }}),
                        Some(ArgInfo { name, r#type: TypeOrOverride::OverrideType{ type_repr, imports, .. }}) => {
                            let imports = imports.iter().collect::<Vec<&String>>();
                            Ok(quote! {
                            ::pyo3_stub_gen::type_info::ArgInfo {
                                name: #name,
                                r#type: || ::pyo3_stub_gen::TypeInfo { name: #type_repr.to_string(), import: ::std::collections::HashSet::from([#(#imports.into(),)*]) },
                                signature: Some(pyo3_stub_gen::type_info::SignatureArg::Args),
                            }})
                        },
                        None => Err(syn::Error::new(ident.span(), format!("can not find argument: {ident}")))
                    }
                },
                SignatureArg::Keywords(_, _, ident) => {
                    let name = ident.to_string();
                    match args_map.get(&name) {
                        Some(ArgInfo { name, r#type: TypeOrOverride::RustType { r#type } }) => Ok(quote! {
                        ::pyo3_stub_gen::type_info::ArgInfo {
                            name: #name,
                            r#type: <#r#type as ::pyo3_stub_gen::PyStubType>::type_input,
                            signature: Some(pyo3_stub_gen::type_info::SignatureArg::Keywords),
                        }}),
                        Some(ArgInfo { name, r#type: TypeOrOverride::OverrideType{ type_repr, imports, .. }}) => {
                            let imports = imports.iter().collect::<Vec<&String>>();
                            Ok(quote! {
                            ::pyo3_stub_gen::type_info::ArgInfo {
                                name: #name,
                                r#type: || ::pyo3_stub_gen::TypeInfo { name: #type_repr.to_string(), import: ::std::collections::HashSet::from([#(#imports.into(),)*]) },
                                signature: Some(pyo3_stub_gen::type_info::SignatureArg::Keywords),
                            }})
                        },
                        None => Err(syn::Error::new(ident.span(), format!("can not find argument: {ident}")))
                    }
                }
            }).collect()
        } else {
            self.args
                .iter()
                .map(|arg| {
                    match arg {
                        ArgInfo { name, r#type: TypeOrOverride::RustType { r#type } } => {
                            let mut ty = r#type.clone();
                            remove_lifetime(&mut ty);
                            Ok(quote! {
                                ::pyo3_stub_gen::type_info::ArgInfo {
                                    name: #name,
                                    r#type: <#ty as ::pyo3_stub_gen::PyStubType>::type_input,
                                    signature: None,
                                }
                            })
                        }
                        ArgInfo { name, r#type: TypeOrOverride::OverrideType{ type_repr, imports, .. }} => {
                            let imports = imports.iter().collect::<Vec<&String>>();
                            Ok(quote! {
                            ::pyo3_stub_gen::type_info::ArgInfo {
                                name: #name,
                                r#type: || ::pyo3_stub_gen::TypeInfo { name: #type_repr.to_string(), import: ::std::collections::HashSet::from([#(#imports.into(),)*]) },
                                signature: None,
                            }})
                        },
                    }
                })
                .collect()
        };
        match arg_infos_res {
            Ok(arg_infos) => tokens.append_all(quote! { &[ #(#arg_infos),* ] }),
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
