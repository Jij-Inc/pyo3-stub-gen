use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Expr, Ident, Result, Token,
};

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

impl ToTokens for SignatureArg {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            SignatureArg::Ident(ident) => tokens.append_all(quote! { #ident }),
            SignatureArg::Assign(ident, _eq, value) => {
                tokens.append_all(quote! { #ident = #value })
            }
            SignatureArg::Star(star) => tokens.append_all(quote! { #star }),
            SignatureArg::Args(star, ident) => tokens.append_all(quote! { #star #ident }),
            SignatureArg::Keywords(star1, star2, ident) => {
                tokens.append_all(quote! { #star1 #star2 #ident })
            }
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

impl ToTokens for Signature {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let sig = self
            .args
            .iter()
            .map(|arg| arg.to_token_stream().to_string())
            .collect::<Vec<String>>()
            .join(", ");
        tokens.append_all(quote! { #sig });
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
