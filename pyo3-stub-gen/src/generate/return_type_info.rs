use crate::TypeInfo;
use std::fmt;

/// Wrapper of [TypeInfo] to implement [fmt::Display] which insert `->` before return type for non-`NoReturn` type.
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnTypeInfo {
    pub r#type: TypeInfo,
}

impl fmt::Display for ReturnTypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !matches!(self.r#type, TypeInfo::NoReturn) {
            write!(f, " -> {}", self.r#type)?;
        }
        Ok(())
    }
}

impl From<TypeInfo> for ReturnTypeInfo {
    fn from(r#type: TypeInfo) -> Self {
        Self { r#type }
    }
}
