use crate::{generate::*, type_info::*, TypeInfo};
use std::fmt;

/// Definition of a class member.
#[derive(Debug, Clone, PartialEq)]
pub struct MemberDef {
    pub name: &'static str,
    pub r#type: TypeInfo,
}

impl From<&MemberInfo> for MemberDef {
    fn from(info: &MemberInfo) -> Self {
        Self {
            name: info.name,
            r#type: (info.r#type)(),
        }
    }
}

impl fmt::Display for MemberDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent = indent();
        writeln!(f, "{indent}{}: {}", self.name, self.r#type)
    }
}
