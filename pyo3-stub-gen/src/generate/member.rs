use crate::{generate::*, type_info::*, TypeInfo};
use std::{collections::HashSet, fmt};

/// Definition of a class member.
#[derive(Debug, Clone)]
pub struct MemberDef {
    pub name: &'static str,
    pub r#type: TypeInfo,
    pub default: Option<&'static std::sync::LazyLock<String>>,
    pub doc: &'static str,
}

impl PartialEq for MemberDef {
    fn eq(&self, other: &Self) -> bool {
        let self_default: Option<&String> = self.default.map(|default| &**default);
        let other_default: Option<&String> = other.default.map(|default| &**default);
        self.name == other.name
            && self.r#type == other.r#type
            && self_default == other_default
            && self.doc == other.doc
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
            default: info.default,
            doc: info.doc,
        }
    }
}

impl fmt::Display for MemberDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent = indent();
        write!(f, "{indent}{}: {}", self.name, self.r#type)?;
        if let Some(default) = self.default {
            let default_str: &String = default;
            write!(f, " = {}", default_str)?;
        }
        writeln!(f)?;
        if !self.doc.is_empty() {
            writeln!(f, r#"{indent}r""""#)?;
            for line in self.doc.lines() {
                writeln!(f, "{indent}{}", line)?;
            }
            writeln!(f, r#"{indent}""""#)?;
        }
        Ok(())
    }
}
