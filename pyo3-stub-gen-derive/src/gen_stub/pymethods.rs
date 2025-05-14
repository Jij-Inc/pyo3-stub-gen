use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Error, ImplItem, ItemImpl, Result, Type};

use super::{MemberInfo, MethodInfo};

#[derive(Debug)]
pub struct PyMethodsInfo {
    struct_id: Type,
    attrs: Vec<MemberInfo>,
    getters: Vec<MemberInfo>,
    setters: Vec<MemberInfo>,
    methods: Vec<MethodInfo>,
}

impl TryFrom<ItemImpl> for PyMethodsInfo {
    type Error = Error;
    fn try_from(item: ItemImpl) -> Result<Self> {
        let struct_id = *item.self_ty.clone();
        let mut attrs = Vec::new();
        let mut getters = Vec::new();
        let mut setters = Vec::new();
        let mut methods = Vec::new();
        for inner in item.items.into_iter() {
            match inner {
                ImplItem::Const(item_const) => {
                    if MemberInfo::is_classattr(&item_const.attrs)? {
                        attrs.push(MemberInfo::new_classattr_const(item_const)?);
                    }
                }
                ImplItem::Fn(item_fn) => {
                    if MemberInfo::is_getter(&item_fn.attrs)? {
                        getters.push(MemberInfo::new_getter(item_fn)?);
                        continue;
                    }
                    if MemberInfo::is_setter(&item_fn.attrs)? {
                        setters.push(MemberInfo::new_setter(item_fn)?);
                        continue;
                    }
                    if MemberInfo::is_classattr(&item_fn.attrs)? {
                        attrs.push(MemberInfo::new_classattr_fn(item_fn)?);
                        continue;
                    }
                    let mut method = MethodInfo::try_from(item_fn)?;
                    method.replace_self(&item.self_ty);
                    methods.push(method);
                }
                _ => continue,
            }
        }
        Ok(Self {
            struct_id,
            attrs,
            getters,
            setters,
            methods,
        })
    }
}

impl ToTokens for PyMethodsInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            struct_id,
            attrs,
            getters,
            setters,
            methods,
        } = self;
        tokens.append_all(quote! {
            ::pyo3_stub_gen::type_info::PyMethodsInfo {
                struct_id: std::any::TypeId::of::<#struct_id>,
                attrs: &[ #(#attrs),* ],
                getters: &[ #(#getters),* ],
                setters: &[ #(#setters),* ],
                methods: &[ #(#methods),* ],
            }
        })
    }
}
