use crate::type_info::*;
use std::fmt;

/// Definition of a Python execption.
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorDef {
    pub name: &'static str,
    pub base: &'static str,
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
