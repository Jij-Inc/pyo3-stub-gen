use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Error, ImplItem, ItemImpl, Result, Type};

use super::{MemberInfo, MethodInfo};

pub struct PyMethodsInfo {
    struct_id: Type,
    getters: Vec<MemberInfo>,
    methods: Vec<MethodInfo>,
}

impl TryFrom<ItemImpl> for PyMethodsInfo {
    type Error = Error;
    fn try_from(item: ItemImpl) -> Result<Self> {
        let struct_id = *item.self_ty.clone();
        let mut getters = Vec::new();
        let mut methods = Vec::new();

        for inner in item.items.into_iter() {
            let ImplItem::Fn(item_fn) = inner else {
                continue;
            };
            if MemberInfo::is_candidate_item(&item_fn)? {
                getters.push(MemberInfo::try_from(item_fn)?);
                continue;
            }

            let mut method = MethodInfo::try_from(item_fn)?;
            method.replace_self(&item.self_ty);
            methods.push(method);
        }
        Ok(Self {
            struct_id,
            getters,
            methods,
        })
    }
}

impl ToTokens for PyMethodsInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            struct_id,
            getters,
            methods,
        } = self;
        tokens.append_all(quote! {
            ::pyo3_stub_gen::type_info::PyMethodsInfo {
                struct_id: std::any::TypeId::of::<#struct_id>,
                getters: &[ #(#getters),* ],
                methods: &[ #(#methods),* ],
            }
        })
    }
}
