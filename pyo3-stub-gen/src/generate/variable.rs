use std::fmt;

use crate::{type_info::PyVariableInfo, TypeInfo};

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDef {
    pub name: &'static str,
    pub type_: TypeInfo,
}

impl From<&PyVariableInfo> for VariableDef {
    fn from(info: &PyVariableInfo) -> Self {
        Self {
            name: info.name,
            type_: (info.type_)(),
        }
    }
}

impl fmt::Display for VariableDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.type_)
    }
}
