use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parse::{Parse, ParseStream},
    Error, FnArg, ItemFn, Result,
};

use crate::gen_stub::util::TypeOrOverride;

use super::{
    attr::IgnoreTarget, extract_deprecated, extract_documents, extract_return_type,
    parameter::Parameters, parse_args, parse_gen_stub_type_ignore, parse_pyo3_attrs, parse_python,
    quote_option, Attr, DeprecatedInfo,
};

pub struct PyFunctionInfo {
    pub(crate) name: String,
    pub(crate) parameters: Parameters,
    pub(crate) r#return: Option<TypeOrOverride>,
    pub(crate) doc: String,
    pub(crate) module: Option<String>,
    pub(crate) is_async: bool,
    pub(crate) deprecated: Option<DeprecatedInfo>,
    pub(crate) type_ignored: Option<IgnoreTarget>,
    pub(crate) is_overload: bool,
    pub(crate) index: usize,
}

#[derive(Default)]
pub(crate) struct PyFunctionAttr {
    pub(crate) module: Option<String>,
    pub(crate) python: Option<syn::LitStr>,
    pub(crate) python_overload: Option<syn::LitStr>,
    pub(crate) no_default_overload: bool,
}

impl Parse for PyFunctionAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut module = None;
        let mut python = None;
        let mut python_overload = None;
        let mut no_default_overload = false;

        // Parse comma-separated key-value pairs
        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;

            match key.to_string().as_str() {
                "module" => {
                    let _: syn::token::Eq = input.parse()?;
                    let value: syn::LitStr = input.parse()?;
                    module = Some(value.value());
                }
                "python" => {
                    let _: syn::token::Eq = input.parse()?;
                    let value: syn::LitStr = input.parse()?;
                    python = Some(value);
                }
                "python_overload" => {
                    let _: syn::token::Eq = input.parse()?;
                    let value: syn::LitStr = input.parse()?;
                    python_overload = Some(value);
                }
                "no_default_overload" => {
                    let _: syn::token::Eq = input.parse()?;
                    let value: syn::LitBool = input.parse()?;
                    no_default_overload = value.value();
                }
                _ => {
                    return Err(Error::new(
                        key.span(),
                        format!("Unknown parameter: {}", key),
                    ));
                }
            }

            // Check for comma separator
            if input.peek(syn::token::Comma) {
                let _: syn::token::Comma = input.parse()?;
            } else {
                break;
            }
        }

        // Validate: cannot mix python and python_overload
        if python.is_some() && python_overload.is_some() {
            return Err(Error::new(
                input.span(),
                "Cannot specify both 'python' and 'python_overload' parameters. Use 'python' for single signatures or 'python_overload' for multiple overloads.",
            ));
        }

        // Validate: no_default_overload requires python_overload
        if no_default_overload && python_overload.is_none() {
            return Err(Error::new(
                input.span(),
                "The 'no_default_overload' parameter can only be used with 'python_overload'. \
                 Use 'python_overload' to define multiple overload signatures.",
            ));
        }

        Ok(Self {
            module,
            python,
            python_overload,
            no_default_overload,
        })
    }
}

impl TryFrom<ItemFn> for PyFunctionInfo {
    type Error = Error;
    fn try_from(item: ItemFn) -> Result<Self> {
        let doc = extract_documents(&item.attrs).join("\n");
        let deprecated = extract_deprecated(&item.attrs);
        let type_ignored = parse_gen_stub_type_ignore(&item.attrs)?;
        let args = parse_args(item.sig.inputs)?;
        let r#return = extract_return_type(&item.sig.output, &item.attrs)?;
        let mut name = None;
        let mut sig = None;
        for attr in parse_pyo3_attrs(&item.attrs)? {
            match attr {
                Attr::Name(function_name) => name = Some(function_name),
                Attr::Signature(signature) => sig = Some(signature),
                _ => {}
            }
        }
        let name = name.unwrap_or_else(|| item.sig.ident.to_string());

        // Build parameters from args and signature
        let parameters = if let Some(sig) = sig {
            Parameters::new_with_sig(&args, &sig)?
        } else {
            Parameters::new(&args)
        };

        Ok(Self {
            name,
            parameters,
            r#return,
            doc,
            module: None,
            is_async: item.sig.asyncness.is_some(),
            deprecated,
            type_ignored,
            is_overload: false, // Default to false, will be set by macro if needed
            index: 0, // Default to 0, will be set by macro if multiple functions are generated
        })
    }
}

impl ToTokens for PyFunctionInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            r#return: ret,
            name,
            doc,
            parameters,
            module,
            is_async,
            deprecated,
            type_ignored,
            is_overload,
            index,
        } = self;
        let ret_tt = if let Some(ret) = ret {
            match ret {
                TypeOrOverride::RustType { r#type } => {
                    let ty = r#type.clone();
                    quote! { <#ty as pyo3_stub_gen::PyStubType>::type_output }
                }
                TypeOrOverride::OverrideType {
                    type_repr, imports, rust_type_markers, ..
                } => {
                    let imports = imports.iter().collect::<Vec<&String>>();

                    // Generate code to process RustType markers
                    let (type_name_code, type_refs_code) = if rust_type_markers.is_empty() {
                        (
                            quote! { #type_repr.to_string() },
                            quote! { ::std::collections::HashMap::new() },
                        )
                    } else {
                        let marker_types: Vec<syn::Type> = rust_type_markers
                            .iter()
                            .filter_map(|s| syn::parse_str(s).ok())
                            .collect();

                        let rust_names = rust_type_markers.iter().collect::<Vec<_>>();

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
                                                type_info.name.split('[').next().unwrap_or(&type_info.name).split('.').last().unwrap_or(&type_info.name).to_string(),
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
                    };

                    quote! {
                        || ::pyo3_stub_gen::TypeInfo { name: #type_name_code, source_module: None, import: ::std::collections::HashSet::from([#(#imports.into(),)*]), type_refs: #type_refs_code }
                    }
                }
            }
        } else {
            quote! { ::pyo3_stub_gen::type_info::no_return_type_output }
        };
        // let sig_tt = quote_option(sig);
        let module_tt = quote_option(module);
        let deprecated_tt = deprecated
            .as_ref()
            .map(|d| quote! { Some(#d) })
            .unwrap_or_else(|| quote! { None });
        let type_ignored_tt = if let Some(target) = type_ignored {
            match target {
                IgnoreTarget::All => {
                    quote! { Some(::pyo3_stub_gen::type_info::IgnoreTarget::All) }
                }
                IgnoreTarget::SpecifiedLits(rules) => {
                    let rule_strs: Vec<String> = rules.iter().map(|lit| lit.value()).collect();
                    quote! {
                        Some(::pyo3_stub_gen::type_info::IgnoreTarget::Specified(
                            &[#(#rule_strs),*] as &[&str]
                        ))
                    }
                }
            }
        } else {
            quote! { None }
        };

        tokens.append_all(quote! {
            ::pyo3_stub_gen::type_info::PyFunctionInfo {
                name: #name,
                parameters: #parameters,
                r#return: #ret_tt,
                doc: #doc,
                module: #module_tt,
                is_async: #is_async,
                deprecated: #deprecated_tt,
                type_ignored: #type_ignored_tt,
                is_overload: #is_overload,
                file: file!(),
                line: line!(),
                column: column!(),
                index: #index,
            }
        })
    }
}

// `#[gen_stub(xxx)]` is not a valid proc_macro_attribute
// it's only designed to receive user's setting.
// We need to remove all `#[gen_stub(xxx)]` before print the item_fn back
pub fn prune_attrs(item_fn: &mut ItemFn) {
    super::attr::prune_attrs(&mut item_fn.attrs);
    for arg in item_fn.sig.inputs.iter_mut() {
        if let FnArg::Typed(ref mut pat_type) = arg {
            super::attr::prune_attrs(&mut pat_type.attrs);
        }
    }
}

/// Represents one or more PyFunctionInfo with the original ItemFn.
/// This handles the case where python_overload generates multiple function signatures.
pub struct PyFunctionInfos {
    pub(crate) item_fn: ItemFn,
    pub(crate) infos: Vec<PyFunctionInfo>,
}

impl PyFunctionInfos {
    /// Create PyFunctionInfos from ItemFn and PyFunctionAttr
    pub fn from_parts(mut item_fn: ItemFn, attr: PyFunctionAttr) -> Result<Self> {
        // Handle python stub syntax early (doesn't need base_info)
        if let Some(python) = attr.python {
            let mut python_info = parse_python::parse_python_function_stub(python)?;
            python_info.module = attr.module;
            prune_attrs(&mut item_fn);
            return Ok(Self {
                item_fn,
                infos: vec![python_info],
            });
        }

        // Convert ItemFn to base PyFunctionInfo for Rust-based generation
        let mut base_info = PyFunctionInfo::try_from(item_fn.clone())?;
        base_info.module = attr.module;

        let infos = if let Some(python_overload) = attr.python_overload {
            // Get function name for validation
            let function_name = base_info.name.clone();

            // Parse multiple overload definitions
            let mut overload_infos =
                parse_python::parse_python_overload_stubs(python_overload, &function_name)?;

            // Preserve module information and assign indices
            for (index, info) in overload_infos.iter_mut().enumerate() {
                info.module = base_info.module.clone();
                info.index = index;
            }

            // If no_default_overload is false (default), also generate from Rust type
            if !attr.no_default_overload {
                // Mark the Rust-generated function as overload
                base_info.is_overload = true;
                base_info.index = overload_infos.len();
                overload_infos.push(base_info);
            }

            overload_infos
        } else {
            // No python or python_overload, use auto-generated
            vec![base_info]
        };

        // Prune attributes from ItemFn
        prune_attrs(&mut item_fn);

        Ok(Self { item_fn, infos })
    }
}

impl ToTokens for PyFunctionInfos {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let item_fn = &self.item_fn;
        let infos = &self.infos;

        // Generate multiple submit! blocks
        let submits = infos.iter().map(|info| {
            quote! {
                #[automatically_derived]
                pyo3_stub_gen::inventory::submit! {
                    #info
                }
            }
        });

        tokens.append_all(quote! {
            #(#submits)*
            #item_fn
        })
    }
}
