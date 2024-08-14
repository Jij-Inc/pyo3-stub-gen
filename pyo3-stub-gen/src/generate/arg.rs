use crate::{generate::Import, stub_type::ModuleRef, type_info::*, TypeInfo};
use std::{collections::HashSet, fmt};

#[derive(Debug, Clone, PartialEq)]
pub struct Arg {
    pub name: &'static str,
    pub r#type: TypeInfo,
}

impl Import for Arg {
    fn import(&self) -> HashSet<ModuleRef> {
        self.r#type.import.clone()
    }
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
