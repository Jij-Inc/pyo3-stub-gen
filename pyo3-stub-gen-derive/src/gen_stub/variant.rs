use crate::gen_stub::arg::ArgInfo;
use crate::gen_stub::attr::{extract_documents, parse_pyo3_attrs, Attr};
use crate::gen_stub::member::MemberInfo;
use crate::gen_stub::parameter::Parameters;
use crate::gen_stub::renaming::RenamingRule;
use crate::gen_stub::signature::Signature;
use crate::gen_stub::util::quote_option;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::spanned::Spanned;
use syn::{Fields, Result, Variant};

#[derive(Debug, Clone, Copy)]
pub enum VariantForm {
    Struct,
    Tuple,
    Unit,
}

impl ToTokens for VariantForm {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let token = match self {
            VariantForm::Struct => Ident::new("Struct", Span::call_site()),
            VariantForm::Tuple => Ident::new("Tuple", Span::call_site()),
            VariantForm::Unit => Ident::new("Unit", Span::call_site()),
        };

        tokens.append(token);
    }
}

pub struct VariantInfo {
    pyclass_name: String,
    module: Option<String>,
    fields: Vec<MemberInfo>,
    doc: String,
    form: VariantForm,
    constr_args: Vec<ArgInfo>,
    constr_sig: Option<Signature>,
}

impl VariantInfo {
    pub fn from_variant(variant: Variant, renaming_rule: &Option<RenamingRule>) -> Result<Self> {
        let Variant {
            ident,
            fields,
            attrs,
            ..
        } = variant;

        let mut pyclass_name = None;
        let mut module = None;
        let mut constr_sig = None;
        for attr in parse_pyo3_attrs(&attrs)? {
            match attr {
                Attr::Name(name) => pyclass_name = Some(name),
                Attr::Module(name) => {
                    module = Some(name);
                }
                Attr::Constructor(sig) => {
                    constr_sig = Some(sig);
                }
                _ => {}
            }
        }

        let mut pyclass_name = pyclass_name.unwrap_or_else(|| ident.to_string());
        if let Some(renaming_rule) = renaming_rule {
            pyclass_name = renaming_rule.apply(&pyclass_name);
        }

        let mut members = Vec::new();

        let form = match fields {
            Fields::Unit => VariantForm::Unit,
            Fields::Named(fields) => {
                for field in fields.named {
                    members.push(MemberInfo::try_from(field)?)
                }
                VariantForm::Struct
            }
            Fields::Unnamed(fields) => {
                for (i, field) in fields.unnamed.iter().enumerate() {
                    let mut named_field = field.clone();
                    named_field.ident = Some(Ident::new(&format!("_{i}"), field.ident.span()));
                    members.push(MemberInfo::try_from(named_field)?)
                }
                VariantForm::Tuple
            }
        };

        let constr_args = members.iter().map(|f| f.clone().into()).collect();

        let doc = extract_documents(&attrs).join("\n");
        Ok(Self {
            pyclass_name,
            fields: members,
            module,
            doc,
            form,
            constr_args,
            constr_sig,
        })
    }
}

impl ToTokens for VariantInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            pyclass_name,
            fields,
            doc,
            module,
            form,
            constr_args,
            constr_sig,
        } = self;

        let parameters = if let Some(sig) = constr_sig {
            match Parameters::new_with_sig(constr_args, sig) {
                Ok(params) => params,
                Err(err) => {
                    tokens.extend(err.to_compile_error());
                    return;
                }
            }
        } else {
            Parameters::new(constr_args)
        };

        let module = quote_option(module);
        tokens.append_all(quote! {
            ::pyo3_stub_gen::type_info::VariantInfo {
                pyclass_name: #pyclass_name,
                fields: &[ #( #fields),* ],
                module: #module,
                doc: #doc,
                form: &pyo3_stub_gen::type_info::VariantForm::#form,
                constr_args: #parameters,
            }
        })
    }
}
