use itertools::Itertools;

use crate::{generate::*, type_info::*, TypeInfo};
use std::{collections::HashSet, fmt};

/// Definition of a class member.
#[derive(Debug, Clone, PartialEq)]
pub struct MemberDef {
    pub name: &'static str,
    pub r#type: TypeInfo,
    pub doc: &'static str,
}

impl MemberDef {
    pub fn as_docs(&self, indent: &str) -> String {
        let Self { name, doc, .. } = self;
        let docs = format!("{name} ({})", &self.r#type.name);
        if doc.len() > 0 {
            format!(
                "{docs}: {}",
                doc.split("\n").join(&format!("\n{indent}{indent}{indent}"))
            )
        } else {
            docs
        }
    }
}

impl Import for MemberDef {
    fn import(&self) -> HashSet<ModuleRef> {
        self.r#type.import.clone()
    }
}

impl From<&MemberInfo> for MemberDef {
    fn from(info: &MemberInfo) -> Self {
        Self {
            name: info.name,
            r#type: (info.r#type)(),
            doc: info.doc,
        }
    }
}

impl fmt::Display for MemberDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent = indent();
        writeln!(f, "{indent}{}: {}", self.name, self.r#type)
    }
}
