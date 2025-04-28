use std::collections::HashMap;

use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Expr, Ident, Result, Token, Type,
};

use crate::gen_stub::remove_lifetime;

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
        let arg_infos: Vec<TokenStream2> = if let Some(sig) = self.sig {
            // record all Type information from rust's args
            let args_map: HashMap<String, Type> = self
                .args
                .iter()
                .map(|arg| {
                    let mut ty = arg.r#type.clone();
                    remove_lifetime(&mut ty);
                    (arg.name.clone(), ty)
                })
                .collect();
            sig.args.iter().map(|sig_arg| match sig_arg {
                SignatureArg::Ident(ident) => {
                    let name = ident.to_string();
                    let ty = args_map.get(&name).unwrap();
                    quote! {
                        ::pyo3_stub_gen::type_info::ArgInfo {
                            name: #name,
                            r#type: <#ty as ::pyo3_stub_gen::PyStubType>::type_input,
                            signature: Some(pyo3_stub_gen::type_info::SignatureArg::Ident),
                        }
                    }
                }
                SignatureArg::Assign(ident, _eq, value) => {
                    let name = ident.to_string();
                    let ty = args_map.get(&name).unwrap();
                    let default = if value.to_token_stream().to_string() == "None" {
                        quote! {
                            "None".to_string()
                        }
                    } else {
                        quote! {
                            ::pyo3::prepare_freethreaded_python();
                            ::pyo3::Python::with_gil(|py| -> String {
                                let v: #ty = #value;
                                if let Ok(py_obj) = <#ty as ::pyo3::IntoPyObjectExt>::into_bound_py_any(v, py) {
                                    ::pyo3_stub_gen::util::fmt_py_obj(&py_obj)
                                } else {
                                    "...".to_owned()
                                }
                            })
                        }
                    };
                    quote! {
                        ::pyo3_stub_gen::type_info::ArgInfo {
                            name: #name,
                            r#type: <#ty as ::pyo3_stub_gen::PyStubType>::type_input,
                            signature: Some(pyo3_stub_gen::type_info::SignatureArg::Assign{
                                default: {
                                    static DEFAULT: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
                                        #default
                                    });
                                    &DEFAULT
                                }
                            }),
                        }
                    }
                },
                SignatureArg::Star(_) => quote! {
                    ::pyo3_stub_gen::type_info::ArgInfo {
                        name: "",
                        r#type: <() as ::pyo3_stub_gen::PyStubType>::type_input,
                        signature: Some(pyo3_stub_gen::type_info::SignatureArg::Star),
                    }
                },
                SignatureArg::Args(_, ident) => {
                    let name = ident.to_string();
                    let ty = args_map.get(&name).unwrap();
                    quote! {
                        ::pyo3_stub_gen::type_info::ArgInfo {
                            name: #name,
                            r#type: <#ty as ::pyo3_stub_gen::PyStubType>::type_input,
                            signature: Some(pyo3_stub_gen::type_info::SignatureArg::Args),
                        }
                    }
                },
                SignatureArg::Keywords(_, _, ident) => {
                    let name = ident.to_string();
                    let ty = args_map.get(&name).unwrap();
                    quote! {
                        ::pyo3_stub_gen::type_info::ArgInfo {
                            name: #name,
                            r#type: <#ty as ::pyo3_stub_gen::PyStubType>::type_input,
                            signature: Some(pyo3_stub_gen::type_info::SignatureArg::Keywords),
                        }
                    }
                }
            }).collect()
        } else {
            self.args
                .iter()
                .map(|arg| {
                    let mut ty = arg.r#type.clone();
                    remove_lifetime(&mut ty);
                    let name = &arg.name;
                    quote! {
                        ::pyo3_stub_gen::type_info::ArgInfo {
                            name: #name,
                            r#type: <#ty as ::pyo3_stub_gen::PyStubType>::type_input,
                            signature: None,
                        }
                    }
                })
                .collect()
        };
        tokens.append_all(quote! { &[ #(#arg_infos),* ] });
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
