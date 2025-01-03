use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Error, ImplItem, ItemImpl, Result, Type};

use super::{quote_option, MemberInfo, MethodInfo, NewInfo};

pub struct PyMethodsInfo {
    struct_id: Type,
    new: Option<NewInfo>,
    getters: Vec<MemberInfo>,
    methods: Vec<MethodInfo>,
}

pub(crate) fn prune_attrs(item: &mut ItemImpl) {
    for inner in item.items.iter_mut() {
        if let ImplItem::Fn(item_fn) = inner {
            item_fn.attrs = std::mem::take(&mut item_fn.attrs)
                .into_iter()
                .filter_map(|attr| {
                    if attr.path().is_ident("gen_stub") {
                        None
                    } else {
                        Some(attr)
                    }
                })
                .collect();
        }
    }
}

impl TryFrom<ItemImpl> for PyMethodsInfo {
    type Error = Error;
    fn try_from(item: ItemImpl) -> Result<Self> {
        let struct_id = *item.self_ty.clone();
        let mut new = None;
        let mut getters = Vec::new();
        let mut methods = Vec::new();

        for inner in item.items {
            if let ImplItem::Fn(item_fn) = inner {
                if NewInfo::is_candidate(&item_fn)? {
                    new = Some(NewInfo::try_from(item_fn)?);
                } else if MemberInfo::is_candidate_item(&item_fn)? {
                    getters.push(MemberInfo::try_from(item_fn)?);
                } else {
                    let mut method = MethodInfo::try_from(item_fn)?;
                    method.replace_self(&item.self_ty);
                    methods.push(method);
                }
            }
        }
        Ok(Self {
            struct_id,
            new,
            getters,
            methods,
        })
    }
}

impl ToTokens for PyMethodsInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            struct_id,
            new,
            getters,
            methods,
        } = self;
        let new_tt = quote_option(new);
        tokens.append_all(quote! {
            ::pyo3_stub_gen::type_info::PyMethodsInfo {
                struct_id: std::any::TypeId::of::<#struct_id>,
                new: #new_tt,
                getters: &[ #(#getters),* ],
                methods: &[ #(#methods),* ],
            }
        })
    }
}
