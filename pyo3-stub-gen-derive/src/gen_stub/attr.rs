use super::Signature;
use proc_macro2::TokenTree;
use quote::ToTokens;
use syn::{Attribute, Expr, ExprLit, Ident, Lit, Meta, MetaList, Result};

pub fn extract_documents(attrs: &[Attribute]) -> Vec<String> {
    let mut docs = Vec::new();
    for attr in attrs {
        // `#[doc = "..."]` case
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(syn::MetaNameValue {
                value:
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(doc), ..
                    }),
                ..
            }) = &attr.meta
            {
                let doc = doc.value();
                // Remove head space
                //
                // ```
                // /// This is special document!
                //    ^ This space is trimmed here
                // ```
                docs.push(if !doc.is_empty() && doc.starts_with(' ') {
                    doc[1..].to_string()
                } else {
                    doc
                });
            }
        }
    }
    docs
}

/// `#[pyo3(...)]` style attributes appear in `#[pyclass]` and `#[pymethods]` proc-macros
///
/// As the reference of PyO3 says:
///
/// https://docs.rs/pyo3/latest/pyo3/attr.pyclass.html
/// > All of these parameters can either be passed directly on the `#[pyclass(...)]` annotation,
/// > or as one or more accompanying `#[pyo3(...)]` annotations,
///
/// `#[pyclass(name = "MyClass", module = "MyModule")]` will be decomposed into
/// `#[pyclass]` + `#[pyo3(name = "MyClass")]` + `#[pyo3(module = "MyModule")]`,
/// i.e. two `Attr`s will be created for this case.
///
#[derive(Debug, Clone, PartialEq)]
pub enum Attr {
    // Attributes appears in `#[pyo3(...)]` form or its equivalence
    Name(String),
    Get,
    GetAll,
    Module(String),
    Signature(Signature),
    SpecifiedSignature(String),

    // Attributes appears in components within `#[pymethods]`
    // <https://docs.rs/pyo3/latest/pyo3/attr.pymethods.html>
    New,
    Getter(Option<String>),
    StaticMethod,
    ClassMethod,
}

pub fn parse_pyo3_attrs(attrs: &[Attribute]) -> Result<Vec<Attr>> {
    let mut out = Vec::new();
    for attr in attrs {
        let mut pyo3_attrs = parse_pyo3_attr(attr)?;
        let mut gen_stub_attrs = parse_gen_stub_attr(attr)?;
        out.append(&mut pyo3_attrs);
        out.append(&mut gen_stub_attrs);
    }
    Ok(out)
}

pub fn parse_pyo3_attr(attr: &Attribute) -> Result<Vec<Attr>> {
    let mut pyo3_attrs = Vec::new();
    let path = attr.path();
    if path.is_ident("pyclass")
        || path.is_ident("pymethods")
        || path.is_ident("pyfunction")
        || path.is_ident("pyo3")
    {
        // Inner tokens of `#[pyo3(...)]` may not be nested meta
        // which can be parsed by `Attribute::parse_nested_meta`
        // due to the case of `#[pyo3(signature = (...))]`.
        // https://pyo3.rs/v0.19.1/function/signature
        if let Meta::List(MetaList { tokens, .. }) = &attr.meta {
            use TokenTree::*;
            let tokens: Vec<TokenTree> = tokens.clone().into_iter().collect();
            // Since `(...)` part with `signature` becomes `TokenTree::Group`,
            // we can split entire stream by `,` first, and then pattern match to each cases.
            for tt in tokens.split(|tt| {
                if let Punct(p) = tt {
                    p.as_char() == ','
                } else {
                    false
                }
            }) {
                match tt {
                    [Ident(ident)] => {
                        if ident == "get" {
                            pyo3_attrs.push(Attr::Get);
                        }
                        if ident == "get_all" {
                            pyo3_attrs.push(Attr::GetAll);
                        }
                    }
                    [Ident(ident), Punct(_), Literal(lit)] => {
                        if ident == "name" {
                            pyo3_attrs
                                .push(Attr::Name(lit.to_string().trim_matches('"').to_string()));
                        }
                        if ident == "module" {
                            pyo3_attrs
                                .push(Attr::Module(lit.to_string().trim_matches('"').to_string()));
                        }
                    }
                    [Ident(ident), Punct(_), Group(group)] => {
                        if ident == "signature" {
                            pyo3_attrs.push(Attr::Signature(syn::parse2(group.to_token_stream())?));
                        }
                    }
                    _ => {}
                }
            }
        }
    } else if path.is_ident("new") {
        pyo3_attrs.push(Attr::New);
    } else if path.is_ident("staticmethod") {
        pyo3_attrs.push(Attr::StaticMethod);
    } else if path.is_ident("classmethod") {
        pyo3_attrs.push(Attr::ClassMethod);
    } else if path.is_ident("getter") {
        if let Ok(inner) = attr.parse_args::<Ident>() {
            pyo3_attrs.push(Attr::Getter(Some(inner.to_string())));
        } else {
            pyo3_attrs.push(Attr::Getter(None));
        }
    }

    Ok(pyo3_attrs)
}

pub fn parse_gen_stub_attr(attr: &Attribute) -> Result<Vec<Attr>> {
    let mut gen_stub_attrs = Vec::new();
    let path = attr.path();
    if path.is_ident("gen_stub") {
        // Inner tokens of `#[pyo3(...)]` may not be nested meta
        // which can be parsed by `Attribute::parse_nested_meta`
        // due to the case of `#[pyo3(signature = (...))]`.
        // https://pyo3.rs/v0.19.1/function/signature
        if let Meta::List(MetaList { tokens, .. }) = &attr.meta {
            use TokenTree::*;
            let tokens: Vec<TokenTree> = tokens.clone().into_iter().collect();
            // Since `(...)` part with `signature` becomes `TokenTree::Group`,
            // we can split entire stream by `,` first, and then pattern match to each cases.
            for tt in tokens.split(|tt| {
                if let Punct(p) = tt {
                    p.as_char() == ','
                } else {
                    false
                }
            }) {
                #[expect(clippy::single_match)] // use match for future's extension
                match tt {
                    [Ident(ident), Punct(_), Group(group)] => {
                        if ident == "signature" {
                            gen_stub_attrs.push(Attr::SpecifiedSignature(
                                group
                                    .to_string()
                                    .replace('\n', "")
                                    .trim_start_matches('(')
                                    .trim_end_matches(')')
                                    .trim_end_matches(',')
                                    .to_string(),
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(gen_stub_attrs)
}

/// To obtain all args' name, we can split by comma.
/// However, to ignore the comma in types, e.g. `tuple[str, str]`,
/// we only adopt the comma in top level, i.e., bracket_depth = 0
fn extract_args_name(params: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut bracket_depth: usize = 0;
    for ch in params.chars() {
        match ch {
            '[' | '(' | '{' => {
                bracket_depth += 1;
                current.push(ch);
            }
            ']' | ')' | '}' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                current.push(ch);
            }
            // if meet comma in top, split and push to current
            ',' if bracket_depth == 0 => {
                let token = current.trim().to_string();
                if !token.is_empty() {
                    result.push(token);
                }
                current.clear();
            }
            // otherwise, directly push
            _ => {
                current.push(ch);
            }
        }
    }
    let token = current.trim().to_string();
    if !token.is_empty() {
        result.push(token);
    }

    result
        .into_iter()
        .map(|token| {
            let mut _token = token.as_str();
            if let Some((new_token, _)) = _token.split_once(':') {
                _token = new_token;
            };
            if let Some((new_token, _)) = _token.split_once('=') {
                _token = new_token;
            };
            if let Some((_, new_token)) = _token.split_once('*') {
                _token = new_token;
            };
            if let Some((_, new_token)) = _token.split_once('*') {
                _token = new_token;
            };
            _token = _token.trim();
            _token.to_string()
        })
        .collect()
}

pub(crate) fn check_specified_signature(
    fn_name: &str,
    specified_sig: &Option<String>,
    args: &[super::ArgInfo],
    text_sig: &Option<Signature>,
) -> Result<()> {
    if let Some(specified_sig) = &specified_sig {
        let specified_args_name = extract_args_name(specified_sig);
        let args_name = if let Some(sig) = text_sig {
            sig.args_name()
        } else {
            args.iter().map(|s| s.name.clone()).collect()
        };
        if specified_args_name != args_name {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("`fn {fn_name}`: Incorrect args name in #[gen_stub(signature = ( `args` ))]. Want {args_name:?}, found {specified_args_name:?}"),
              ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use syn::{parse_str, Fields, ItemStruct};

    #[test]
    fn test_parse_pyo3_attr() -> Result<()> {
        let item: ItemStruct = parse_str(
            r#"
            #[pyclass(mapping, module = "my_module", name = "Placeholder")]
            pub struct PyPlaceholder {
                #[pyo3(get)]
                pub name: String,
            }
            "#,
        )?;
        // `#[pyclass]` part
        let attrs = parse_pyo3_attr(&item.attrs[0])?;
        assert_eq!(
            attrs,
            vec![
                Attr::Module("my_module".to_string()),
                Attr::Name("Placeholder".to_string())
            ]
        );

        // `#[pyo3(get)]` part
        if let Fields::Named(fields) = item.fields {
            let attrs = parse_pyo3_attr(&fields.named[0].attrs[0])?;
            assert_eq!(attrs, vec![Attr::Get]);
        } else {
            unreachable!()
        }
        Ok(())
    }

    #[test]
    fn test_args_name() {
        assert_eq!(
            extract_args_name(
                "name:tuple[str, str] | None = None, *inputs, outputs=None, **options"
            ),
            vec!["name", "inputs", "outputs", "options"]
        );
    }
}
