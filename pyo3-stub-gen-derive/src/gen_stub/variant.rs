use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{ToTokens};
use syn::{Fields, Result, Variant};
use crate::gen_stub::member::MemberInfo;
use crate::gen_stub::pyclass_tree::PyClassTreeInfo;
use crate::gen_stub::renaming::RenamingRule;

pub enum VariantInfo {
    Tuple {
        tuple: MemberInfo
    },
    Struct {
        pyclass: PyClassTreeInfo
    }
}

impl From<PyClassTreeInfo> for VariantInfo {
    fn from(pyclass: PyClassTreeInfo) -> Self {
        VariantInfo::Struct { pyclass }
    }
}

impl From<MemberInfo> for VariantInfo {
    fn from(member: MemberInfo) -> Self {
        VariantInfo::Tuple { tuple: member }
    }
}

impl VariantInfo {
    pub fn from_variant(variant: Variant, r#enum: Ident, renaming_rule: &Option<RenamingRule>) -> Result<Self> {
        match &variant.fields {
            Fields::Unit |
            Fields::Named(_) => {
                Ok(PyClassTreeInfo::from_variant(variant, r#enum, renaming_rule)?.into())
            }
            Fields::Unnamed(_) => {
                Ok(MemberInfo::from_variant(variant, r#enum, renaming_rule)?.into())
            }
        }

    }
}

impl ToTokens for VariantInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            VariantInfo::Tuple { tuple } => {
                tuple.to_tokens(tokens);
            }
            VariantInfo::Struct { pyclass } => {
                pyclass.to_tokens(tokens);
            }
        }


    }
}