use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::Type;

pub struct StubType {
    pub(crate) ty: Type,
    pub(crate) name: String,
    pub(crate) module: Option<String>,
}

impl ToTokens for StubType {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self { ty, name, module } = self;
        let module_tt = if let Some(module) = module {
            quote! { #module.into() }
        } else {
            quote! { Default::default() }
        };

        tokens.append_all(quote! {
            #[automatically_derived]
            impl ::pyo3_stub_gen::PyStubType for #ty {
                fn type_output() -> ::pyo3_stub_gen::TypeInfo {
                    ::pyo3_stub_gen::TypeInfo::locally_defined(#name, #module_tt)
                }
            }
        })
    }
}
