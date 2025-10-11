use indexmap::IndexSet;

use super::{RenamingRule, Signature};
use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Attribute, Expr, ExprLit, Ident, Lit, LitStr, Meta, MetaList, Result, Token, Type,
};

/// Represents the target of type ignore comments during parsing
#[derive(Debug, Clone, PartialEq)]
pub enum IgnoreTarget {
    /// Ignore all type checking errors `(# type: ignore)`
    All,
    /// Ignore specific type checking rules (stored as LitStr during parsing)
    SpecifiedLits(Vec<LitStr>),
}

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

/// Extract `#[deprecated(...)]` attribute
pub fn extract_deprecated(attrs: &[Attribute]) -> Option<DeprecatedInfo> {
    for attr in attrs {
        if attr.path().is_ident("deprecated") {
            if let Ok(list) = attr.meta.require_list() {
                let mut since = None;
                let mut note = None;

                list.parse_nested_meta(|meta| {
                    if meta.path.is_ident("since") {
                        let value = meta.value()?;
                        let lit: LitStr = value.parse()?;
                        since = Some(lit.value());
                    } else if meta.path.is_ident("note") {
                        let value = meta.value()?;
                        let lit: LitStr = value.parse()?;
                        note = Some(lit.value());
                    }
                    Ok(())
                })
                .ok()?;

                return Some(DeprecatedInfo { since, note });
            }
        }
    }
    None
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
pub struct DeprecatedInfo {
    pub since: Option<String>,
    pub note: Option<String>,
}

impl ToTokens for DeprecatedInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let since = self
            .since
            .as_ref()
            .map(|s| quote! { Some(#s) })
            .unwrap_or_else(|| quote! { None });
        let note = self
            .note
            .as_ref()
            .map(|n| quote! { Some(#n) })
            .unwrap_or_else(|| quote! { None });
        tokens.append_all(quote! {
            ::pyo3_stub_gen::type_info::DeprecatedInfo {
                since: #since,
                note: #note,
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
#[expect(clippy::enum_variant_names)]
pub enum Attr {
    // Attributes appears in `#[pyo3(...)]` form or its equivalence
    Name(String),
    Get,
    GetAll,
    Set,
    SetAll,
    Module(String),
    Constructor(Signature),
    Signature(Signature),
    RenameAll(RenamingRule),
    Extends(Type),

    // Comparison and special method attributes for pyclass
    Eq,
    Ord,
    Hash,
    Str,
    Subclass,

    // Attributes appears in components within `#[pymethods]`
    // <https://docs.rs/pyo3/latest/pyo3/attr.pymethods.html>
    New,
    Getter(Option<String>),
    Setter(Option<String>),
    StaticMethod,
    ClassMethod,
    ClassAttr,
}

pub fn parse_pyo3_attrs(attrs: &[Attribute]) -> Result<Vec<Attr>> {
    let mut out = Vec::new();
    for attr in attrs {
        let mut new = parse_pyo3_attr(attr)?;
        out.append(&mut new);
    }
    Ok(out)
}

pub fn parse_pyo3_attr(attr: &Attribute) -> Result<Vec<Attr>> {
    let mut pyo3_attrs = Vec::new();
    let path = attr.path();
    let is_full_path_pyo3_attr = path.segments.len() == 2
        && path
            .segments
            .first()
            .is_some_and(|seg| seg.ident.eq("pyo3"))
        && path.segments.last().is_some_and(|seg| {
            seg.ident.eq("pyclass") || seg.ident.eq("pymethods") || seg.ident.eq("pyfunction")
        });
    if path.is_ident("pyclass")
        || path.is_ident("pymethods")
        || path.is_ident("pyfunction")
        || path.is_ident("pyo3")
        || is_full_path_pyo3_attr
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
                        if ident == "set" {
                            pyo3_attrs.push(Attr::Set);
                        }
                        if ident == "set_all" {
                            pyo3_attrs.push(Attr::SetAll);
                        }
                        if ident == "eq" {
                            pyo3_attrs.push(Attr::Eq);
                        }
                        if ident == "ord" {
                            pyo3_attrs.push(Attr::Ord);
                        }
                        if ident == "hash" {
                            pyo3_attrs.push(Attr::Hash);
                        }
                        if ident == "str" {
                            pyo3_attrs.push(Attr::Str);
                        }
                        if ident == "subclass" {
                            pyo3_attrs.push(Attr::Subclass);
                        }
                        // frozen is required by PyO3 when using hash, but doesn't affect stub generation
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
                        if ident == "rename_all" {
                            let name = lit.to_string().trim_matches('"').to_string();
                            if let Some(renaming_rule) = RenamingRule::try_new(&name) {
                                pyo3_attrs.push(Attr::RenameAll(renaming_rule));
                            }
                        }
                    }
                    [Ident(ident), Punct(_), Group(group)] => {
                        if ident == "signature" {
                            pyo3_attrs.push(Attr::Signature(syn::parse2(group.to_token_stream())?));
                        } else if ident == "constructor" {
                            pyo3_attrs
                                .push(Attr::Constructor(syn::parse2(group.to_token_stream())?));
                        }
                    }
                    [Ident(ident), Punct(_), Ident(ident2)] => {
                        if ident == "extends" {
                            pyo3_attrs.push(Attr::Extends(syn::parse2(ident2.to_token_stream())?));
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
    } else if path.is_ident("classattr") {
        pyo3_attrs.push(Attr::ClassAttr);
    } else if path.is_ident("getter") {
        if let Ok(inner) = attr.parse_args::<Ident>() {
            pyo3_attrs.push(Attr::Getter(Some(inner.to_string())));
        } else {
            pyo3_attrs.push(Attr::Getter(None));
        }
    } else if path.is_ident("setter") {
        if let Ok(inner) = attr.parse_args::<Ident>() {
            pyo3_attrs.push(Attr::Setter(Some(inner.to_string())));
        } else {
            pyo3_attrs.push(Attr::Setter(None));
        }
    }

    Ok(pyo3_attrs)
}

#[derive(Debug, Clone, PartialEq)]
pub enum StubGenAttr {
    /// Default value for getter
    Default(Expr),
    /// Skip a function in #[pymethods]
    Skip,
    /// Override the python type for a function argument or return type
    OverrideType(OverrideTypeAttribute),
    /// Type checker rules to ignore for this function/method
    TypeIgnore(IgnoreTarget),
}

pub fn prune_attrs(attrs: &mut Vec<Attribute>) {
    attrs.retain(|attr| !attr.path().is_ident("gen_stub"));
}

pub fn parse_gen_stub_override_type(attrs: &[Attribute]) -> Result<Option<OverrideTypeAttribute>> {
    for attr in parse_gen_stub_attrs(attrs, AttributeLocation::Argument, None)? {
        if let StubGenAttr::OverrideType(attr) = attr {
            return Ok(Some(attr));
        }
    }
    Ok(None)
}

pub fn parse_gen_stub_override_return_type(
    attrs: &[Attribute],
) -> Result<Option<OverrideTypeAttribute>> {
    for attr in parse_gen_stub_attrs(attrs, AttributeLocation::Function, None)? {
        if let StubGenAttr::OverrideType(attr) = attr {
            return Ok(Some(attr));
        }
    }
    Ok(None)
}

pub fn parse_gen_stub_default(attrs: &[Attribute]) -> Result<Option<Expr>> {
    for attr in parse_gen_stub_attrs(attrs, AttributeLocation::Function, None)? {
        if let StubGenAttr::Default(default) = attr {
            return Ok(Some(default));
        }
    }
    Ok(None)
}
pub fn parse_gen_stub_skip(attrs: &[Attribute]) -> Result<bool> {
    let skip = parse_gen_stub_attrs(
        attrs,
        AttributeLocation::Field,
        Some(&["override_return_type", "default"]),
    )?
    .iter()
    .any(|attr| matches!(attr, StubGenAttr::Skip));
    Ok(skip)
}

pub fn parse_gen_stub_type_ignore(attrs: &[Attribute]) -> Result<Option<IgnoreTarget>> {
    // Try Function location first (for regular functions)
    for attr in parse_gen_stub_attrs(attrs, AttributeLocation::Function, None)? {
        if let StubGenAttr::TypeIgnore(target) = attr {
            return Ok(Some(target));
        }
    }
    // Try Field location (for methods in #[pymethods] blocks)
    for attr in parse_gen_stub_attrs(attrs, AttributeLocation::Field, None)? {
        if let StubGenAttr::TypeIgnore(target) = attr {
            return Ok(Some(target));
        }
    }
    Ok(None)
}

fn parse_gen_stub_attrs(
    attrs: &[Attribute],
    location: AttributeLocation,
    ignored_idents: Option<&[&str]>,
) -> Result<Vec<StubGenAttr>> {
    let mut out = Vec::new();
    for attr in attrs {
        let mut new = parse_gen_stub_attr(attr, location, ignored_idents.unwrap_or(&[]))?;
        out.append(&mut new);
    }
    Ok(out)
}

fn parse_gen_stub_attr(
    attr: &Attribute,
    location: AttributeLocation,
    ignored_idents: &[&str],
) -> Result<Vec<StubGenAttr>> {
    let mut gen_stub_attrs = Vec::new();
    let path = attr.path();
    if path.is_ident("gen_stub") {
        attr.parse_args_with(|input: ParseStream| {
            while !input.is_empty() {
                let ident: Ident = input.parse()?;
                let ignored_ident = ignored_idents.iter().any(|other| ident == other);
                if (ident == "override_type"
                    && (location == AttributeLocation::Argument || ignored_ident))
                    || (ident == "override_return_type"
                        && (location == AttributeLocation::Function || location == AttributeLocation::Field || ignored_ident))
                {
                    let content;
                    parenthesized!(content in input);
                    let override_attr: OverrideTypeAttribute = content.parse()?;
                    gen_stub_attrs.push(StubGenAttr::OverrideType(override_attr));
                } else if ident == "skip" && (location == AttributeLocation::Field || ignored_ident)
                {
                    gen_stub_attrs.push(StubGenAttr::Skip);
                } else if ident == "default"
                    && input.peek(Token![=])
                    && (location == AttributeLocation::Field || location == AttributeLocation::Function || ignored_ident)
                {
                    input.parse::<Token![=]>()?;
                    gen_stub_attrs.push(StubGenAttr::Default(input.parse()?));
                } else if ident == "type_ignore"
                    && (location == AttributeLocation::Function || location == AttributeLocation::Field || ignored_ident)
                {
                    // Handle two cases:
                    // 1. type_ignore (without equals) -> IgnoreTarget::All
                    // 2. type_ignore = [...] -> IgnoreTarget::Specified(rules)
                    if input.peek(Token![=]) {
                        input.parse::<Token![=]>()?;
                        // Parse array of rule names
                        let content;
                        syn::bracketed!(content in input);
                        let rules = Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?;

                        // Validate: empty Specified should be an error
                        if rules.is_empty() {
                            return Err(syn::Error::new(
                                ident.span(),
                                "type_ignore with empty array is not allowed. Use type_ignore without equals for catch-all, or specify rules in the array."
                            ));
                        }

                        // Store the rules as LitStr for now, will be converted to strings during code generation
                        let rule_lits: Vec<LitStr> = rules.into_iter().collect();
                        gen_stub_attrs.push(StubGenAttr::TypeIgnore(IgnoreTarget::SpecifiedLits(rule_lits)));
                    } else {
                        // No equals sign means catch-all
                        gen_stub_attrs.push(StubGenAttr::TypeIgnore(IgnoreTarget::All));
                    }
                } else if ident == "override_type" {
                    return Err(syn::Error::new(
                        ident.span(),
                        "`override_type(...)` is only valid in argument position".to_string(),
                    ));
                } else if ident == "override_return_type" {
                    return Err(syn::Error::new(
                        ident.span(),
                        "`override_return_type(...)` is only valid in function or method position"
                            .to_string(),
                    ));
                } else if ident == "skip" {
                    return Err(syn::Error::new(
                        ident.span(),
                        "`skip` is only valid in field position".to_string(),
                    ));
                } else if ident == "default" {
                    return Err(syn::Error::new(
                        ident.span(),
                        "`default=xxx` is only valid in field or function position".to_string(),
                    ));
                } else if ident == "type_ignore" {
                    return Err(syn::Error::new(
                        ident.span(),
                        "`type_ignore` or `type_ignore=[...]` is only valid in function or method position".to_string(),
                    ));
                } else if location == AttributeLocation::Argument {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("Unsupported keyword `{ident}`, valid is `override_type(...)`"),
                    ));
                } else if location == AttributeLocation::Field {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("Unsupported keyword `{ident}`, valid is `default=xxx`, `skip`, `override_return_type(...)`, `type_ignore`, or `type_ignore=[...]`"),
                    ));
                } else if location == AttributeLocation::Function {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!(
                            "Unsupported keyword `{ident}`, valid is `default=xxx`, `override_return_type(...)`, `type_ignore`, or `type_ignore=[...]`"
                        ),
                    ));
                } else {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("Unsupported keyword `{ident}`"),
                    ));
                }
                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                } else {
                    break;
                }
            }
            Ok(())
        })?;
    }
    Ok(gen_stub_attrs)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum AttributeLocation {
    Argument,
    Field,
    Function,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverrideTypeAttribute {
    pub(crate) type_repr: String,
    pub(crate) imports: IndexSet<String>,
}

mod kw {
    syn::custom_keyword!(type_repr);
    syn::custom_keyword!(imports);
    syn::custom_keyword!(override_type);
}

impl Parse for OverrideTypeAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut type_repr = None;
        let mut imports = IndexSet::new();

        while !input.is_empty() {
            let lookahead = input.lookahead1();

            if lookahead.peek(kw::type_repr) {
                input.parse::<kw::type_repr>()?;
                input.parse::<Token![=]>()?;
                type_repr = Some(input.parse::<LitStr>()?);
            } else if lookahead.peek(kw::imports) {
                input.parse::<kw::imports>()?;
                input.parse::<Token![=]>()?;

                let content;
                parenthesized!(content in input);
                let parsed_imports = Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?;
                imports = parsed_imports.into_iter().collect();
            } else {
                return Err(lookahead.error());
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(OverrideTypeAttribute {
            type_repr: type_repr
                .ok_or_else(|| input.error("missing type_repr"))?
                .value(),
            imports: imports.iter().map(|i| i.value()).collect(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use syn::{parse_str, Fields, ItemFn, ItemStruct, PatType};

    #[test]
    fn test_parse_pyo3_attr() -> Result<()> {
        let item: ItemStruct = parse_str(
            r#"
            #[pyclass(mapping, module = "my_module", name = "Placeholder")]
            #[pyo3(rename_all = "SCREAMING_SNAKE_CASE")]
            pub struct PyPlaceholder {
                #[pyo3(get)]
                pub name: String,
            }
            "#,
        )?;
        // `#[pyclass]` part
        let attrs = parse_pyo3_attrs(&item.attrs)?;
        assert_eq!(
            attrs,
            vec![
                Attr::Module("my_module".to_string()),
                Attr::Name("Placeholder".to_string()),
                Attr::RenameAll(RenamingRule::ScreamingSnakeCase),
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
    fn test_parse_pyo3_attr_full_path() -> Result<()> {
        let item: ItemStruct = parse_str(
            r#"
            #[pyo3::pyclass(mapping, module = "my_module", name = "Placeholder")]
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
    fn test_parse_gen_stub_field_attr() -> Result<()> {
        let item: ItemStruct = parse_str(
            r#"
            pub struct PyPlaceholder {
                #[gen_stub(default = String::from("foo"), skip)]
                pub field0: String,
                #[gen_stub(skip)]
                pub field1: String,
                #[gen_stub(default = 1+2)]
                pub field2: usize,
            }
            "#,
        )?;
        let fields: Vec<_> = item.fields.into_iter().collect();
        let field0_attrs = parse_gen_stub_attrs(&fields[0].attrs, AttributeLocation::Field, None)?;
        if let StubGenAttr::Default(expr) = &field0_attrs[0] {
            assert_eq!(
                expr.to_token_stream().to_string(),
                "String :: from (\"foo\")"
            );
        } else {
            panic!("attr should be Default");
        };
        assert_eq!(&StubGenAttr::Skip, &field0_attrs[1]);
        let field1_attrs = parse_gen_stub_attrs(&fields[1].attrs, AttributeLocation::Field, None)?;
        assert_eq!(vec![StubGenAttr::Skip], field1_attrs);
        let field2_attrs = parse_gen_stub_attrs(&fields[2].attrs, AttributeLocation::Field, None)?;
        if let StubGenAttr::Default(expr) = &field2_attrs[0] {
            assert_eq!(expr.to_token_stream().to_string(), "1 + 2");
        } else {
            panic!("attr should be Default");
        };
        Ok(())
    }
    #[test]
    fn test_parse_gen_stub_override_type_attr() -> Result<()> {
        let item: ItemFn = parse_str(
            r#"
            #[gen_stub_pyfunction]
            #[pyfunction]
            #[gen_stub(override_return_type(type_repr="typing.Never", imports=("typing")))]
            fn say_hello_forever<'a>(
                #[gen_stub(override_type(type_repr="collections.abc.Callable[[str]]", imports=("collections.abc")))]
                cb: Bound<'a, PyAny>,
            ) -> PyResult<()> {
                loop {
                    cb.call1(("Hello!",))?;
                }
            }
            "#,
        )?;
        let fn_attrs = parse_gen_stub_attrs(&item.attrs, AttributeLocation::Function, None)?;
        assert_eq!(fn_attrs.len(), 1);
        if let StubGenAttr::OverrideType(expr) = &fn_attrs[0] {
            assert_eq!(
                *expr,
                OverrideTypeAttribute {
                    type_repr: "typing.Never".into(),
                    imports: IndexSet::from(["typing".into()])
                }
            );
        } else {
            panic!("attr should be OverrideType");
        };
        if let syn::FnArg::Typed(PatType { attrs, .. }) = &item.sig.inputs[0] {
            let arg_attrs = parse_gen_stub_attrs(attrs, AttributeLocation::Argument, None)?;
            assert_eq!(arg_attrs.len(), 1);
            if let StubGenAttr::OverrideType(expr) = &arg_attrs[0] {
                assert_eq!(
                    *expr,
                    OverrideTypeAttribute {
                        type_repr: "collections.abc.Callable[[str]]".into(),
                        imports: IndexSet::from(["collections.abc".into()])
                    }
                );
            } else {
                panic!("attr should be OverrideType");
            };
        }
        Ok(())
    }
}
