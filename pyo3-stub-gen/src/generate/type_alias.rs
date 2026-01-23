use std::{collections::HashSet, fmt};

use crate::{
    generate::Import,
    stub_type::{ImportRef, ModuleRef, TypeRef},
    type_info::TypeAliasInfo,
    TypeInfo,
};

#[derive(Debug, Clone, PartialEq)]
pub struct TypeAliasDef {
    pub name: &'static str,
    pub type_: TypeInfo,
}

impl From<&TypeAliasInfo> for TypeAliasDef {
    fn from(info: &TypeAliasInfo) -> Self {
        Self {
            name: info.name,
            type_: (info.r#type)(),
        }
    }
}

impl Import for TypeAliasDef {
    fn import(&self) -> HashSet<ImportRef> {
        let mut imports = self.type_.import.clone();
        // Always import TypeAlias from typing
        imports.insert(ImportRef::Type(TypeRef {
            module: ModuleRef::Named("typing".to_string()),
            name: "TypeAlias".to_string(),
        }));
        imports
    }
}

impl fmt::Display for TypeAliasDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: TypeAlias = {}", self.name, self.type_)
    }
}

impl TypeAliasDef {
    /// Format type alias with module-qualified type names
    pub fn fmt_for_module(&self, target_module: &str, f: &mut fmt::Formatter) -> fmt::Result {
        let qualified_type = self.type_.qualified_for_module(target_module);
        write!(f, "{}: TypeAlias = {}", self.name, qualified_type)
    }
}
