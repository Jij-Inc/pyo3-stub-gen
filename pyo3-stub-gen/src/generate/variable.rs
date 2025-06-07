use std::fmt;

use crate::{generate::*, type_info::PyVariableInfo, TypeInfo};

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDef {
    pub name: &'static str,
    pub type_: TypeInfo,
}

impl Build for PyVariableInfo {
    type DefType = VariableDef;

    fn build(&self, current_module_name: &str) -> Self::DefType {
        Self::DefType {
            name: self.name,
            type_: self.r#type.build(current_module_name),
        }
    }
}

impl fmt::Display for VariableDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.type_)
    }
}
