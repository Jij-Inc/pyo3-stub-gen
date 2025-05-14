use crate::{generate::*, type_info::*, TypeInfo};
use std::{
    borrow::Cow,
    collections::HashSet,
    fmt::{self},
};

/// Definition of a class member.
#[derive(Debug, Clone, PartialEq)]
pub struct MemberDef {
    pub name: &'static str,
    pub r#type: TypeInfo,
    pub doc: &'static str,
    pub default: Option<&'static str>,
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
            default: info.default.map(|s| s.as_str()),
        }
    }
}

impl fmt::Display for MemberDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent = indent();
        write!(f, "{indent}{}: {}", self.name, self.r#type)?;
        if let Some(default) = self.default {
            write!(f, " = {default}")?;
        }
        writeln!(f)?;
        docstring::write_docstring(f, self.doc, indent)?;
        Ok(())
    }
}

pub struct GetterDisplay<'a>(pub &'a MemberDef);
pub struct SetterDisplay<'a>(pub &'a MemberDef);

impl fmt::Display for GetterDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent = indent();
        write!(
            f,
            "{indent}@property\n{indent}def {}(self) -> {}:",
            self.0.name, self.0.r#type
        )?;
        let doc = if let Some(default) = self.0.default {
            Cow::Owned(format!("{}\ndefault = {default}", self.0.doc))
        } else {
            Cow::Borrowed(self.0.doc)
        };
        if !doc.is_empty() {
            writeln!(f)?;
            let double_indent = format!("{indent}{indent}");
            docstring::write_docstring(f, &doc, &double_indent)
        } else {
            writeln!(f, " ...")
        }
    }
}

impl fmt::Display for SetterDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent = indent();
        write!(
            f,
            "{indent}@{}.setter\n{indent}def {}(self, value: {}) -> None:",
            self.0.name, self.0.name, self.0.r#type
        )?;
        let doc = if let Some(default) = self.0.default {
            Cow::Owned(format!("{}\ndefault = {default}", self.0.doc))
        } else {
            Cow::Borrowed(self.0.doc)
        };
        if !doc.is_empty() {
            writeln!(f)?;
            let double_indent = format!("{indent}{indent}");
            docstring::write_docstring(f, &doc, &double_indent)
        } else {
            writeln!(f, " ...")
        }
    }
}
