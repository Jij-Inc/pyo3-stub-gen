use crate::type_info::*;
use std::fmt;

/// Definition of a Python exception.
#[derive(Debug, Clone, PartialEq, getset::Getters)]
pub struct ErrorDef {
    #[getset(get = "pub")]
    name: &'static str,
    #[getset(get = "pub")]
    base: &'static str,
}

impl From<&PyErrorInfo> for ErrorDef {
    fn from(info: &PyErrorInfo) -> Self {
        Self {
            name: info.name,
            base: (info.base)(),
        }
    }
}

impl fmt::Display for ErrorDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "class {}({}): ...", self.name, self.base)
    }
}
