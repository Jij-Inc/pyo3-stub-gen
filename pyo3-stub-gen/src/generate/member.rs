use crate::{generate::*, type_info::*, TypeInfo};
use std::{
    collections::HashSet,
    fmt::{self},
};

/// Definition of a class member.
#[derive(Debug, Clone, PartialEq)]
pub struct MemberDef {
    pub name: &'static str,
    pub r#type: TypeInfo,
    pub doc: &'static str,
}

impl Import for MemberDef {
    fn import(&self) -> HashSet<ModuleRef> {
        self.r#type.import.clone()
    }
}

impl Build for MemberInfo {
    type DefType = MemberDef;

    fn build(&self, current_module_name: &str) -> Self::DefType {
        Self::DefType {
            name: self.name,
            r#type: self.r#type.build(current_module_name),
            doc: self.doc,
        }
    }
}

impl fmt::Display for MemberDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent = indent();
        writeln!(f, "{indent}{}: {}", self.name, self.r#type)?;
        docstring::write_docstring(f, self.doc, indent)?;
        Ok(())
    }
}
