use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Expr, Ident, Result, Token,
};

use super::{parameter::*, ArgInfo};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SignatureArg {
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

impl Signature {
    /// Access signature arguments
    pub(crate) fn args(&self) -> impl Iterator<Item = &SignatureArg> {
        self.args.iter()
    }
}

impl Parse for Signature {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let paren = parenthesized!(content in input);
        let args = content.parse_terminated(SignatureArg::parse, Token![,])?;
        Ok(Self { paren, args })
    }
}

/// Arguments with signature information, structured by parameter kinds
pub struct ArgsWithSignature<'a> {
    pub args: &'a Vec<ArgInfo>,
    pub sig: &'a Option<Signature>,
}

impl ToTokens for ArgsWithSignature<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let params_result = if let Some(sig) = self.sig {
            Parameters::new_with_sig(self.args, sig)
        } else {
            Ok(Parameters::new(self.args))
        };

        match params_result {
            Ok(parameters) => parameters.to_tokens(tokens),
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
