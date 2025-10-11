use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Error, FnArg, ImplItem, ItemImpl, Result, Type};

use super::{attr::parse_gen_stub_skip, MemberInfo, MethodInfo};

#[derive(Debug)]
pub struct PyMethodsInfo {
    pub(crate) struct_id: Type,
    pub(crate) attrs: Vec<MemberInfo>,
    pub(crate) getters: Vec<MemberInfo>,
    pub(crate) setters: Vec<MemberInfo>,
    pub(crate) methods: Vec<MethodInfo>,
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
                    if parse_gen_stub_skip(&item_const.attrs)? {
                        continue;
                    }
                    if MemberInfo::is_classattr(&item_const.attrs)? {
                        attrs.push(MemberInfo::new_classattr_const(item_const)?);
                    }
                }
                ImplItem::Fn(item_fn) => {
                    if parse_gen_stub_skip(&item_fn.attrs)? {
                        continue;
                    }
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

// `#[gen_stub(xxx)]` is not a valid proc_macro_attribute
// it's only designed to receive user's setting.
// We need to remove all `#[gen_stub(xxx)]` before print the item_impl back
pub fn prune_attrs(item_impl: &mut ItemImpl) {
    super::attr::prune_attrs(&mut item_impl.attrs);
    for inner in item_impl.items.iter_mut() {
        if let ImplItem::Fn(item_fn) = inner {
            super::attr::prune_attrs(&mut item_fn.attrs);
            for arg in item_fn.sig.inputs.iter_mut() {
                if let FnArg::Typed(ref mut pat_type) = arg {
                    super::attr::prune_attrs(&mut pat_type.attrs);
                }
            }
        }
    }
}
