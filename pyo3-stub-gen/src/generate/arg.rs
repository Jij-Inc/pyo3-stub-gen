use crate::type_info::*;
use pyo3::inspect::types::TypeInfo;
use std::fmt;

#[derive(Debug, Clone, PartialEq, getset::Getters)]
pub struct Arg {
    #[getset(get = "pub")]
    name: &'static str,
    #[getset(get = "pub")]
    r#type: TypeInfo,
}

impl From<&ArgInfo> for Arg {
    fn from(info: &ArgInfo) -> Self {
        Self {
            name: info.name,
            r#type: (info.r#type)(),
        }
    }
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.name, self.r#type)
    }
}
