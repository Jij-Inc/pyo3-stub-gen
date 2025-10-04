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
    pub default: Option<String>,
    pub deprecated: Option<DeprecatedInfo>,
}

impl Import for MemberDef {
    fn import(&self) -> HashSet<ImportRef> {
        let mut import = self.r#type.import.clone();
        // Add typing_extensions import if deprecated
        if self.deprecated.is_some() {
            import.insert("typing_extensions".into());
        }
        import
    }
}

impl From<&MemberInfo> for MemberDef {
    fn from(info: &MemberInfo) -> Self {
        Self {
            name: info.name,
            r#type: (info.r#type)(),
            doc: info.doc,
            default: info.default.map(|f| f()),
            deprecated: info.deprecated.clone(),
        }
    }
}

impl fmt::Display for MemberDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent = indent();
        // Constants cannot have deprecated decorators in Python syntax
        // Log a warning if deprecated is present but will be ignored
        if let Some(_deprecated) = &self.deprecated {
            log::warn!(
                "Ignoring #[deprecated] on constant '{}': Python constants cannot have decorators. \
                Consider using a function instead if deprecation is needed.",
                self.name
            );
        }
        write!(f, "{indent}{}: {}", self.name, self.r#type)?;
        if let Some(default) = &self.default {
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
        // Add deprecated decorator if present
        if let Some(deprecated) = &self.0.deprecated {
            writeln!(f, "{indent}{deprecated}")?;
        }
        write!(
            f,
            "{indent}@property\n{indent}def {}(self) -> {}:",
            self.0.name, self.0.r#type
        )?;
        let doc = if let Some(default) = &self.0.default {
            if default == "..." {
                Cow::Borrowed(self.0.doc)
            } else {
                Cow::Owned(format!(
                    "{}\n```python\ndefault = {default}\n```",
                    self.0.doc
                ))
            }
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
        // Write setter decorator first, then deprecated decorator
        writeln!(f, "{indent}@{}.setter", self.0.name)?;
        if let Some(deprecated) = &self.0.deprecated {
            writeln!(f, "{indent}{deprecated}")?;
        }
        write!(
            f,
            "{indent}def {}(self, value: {}) -> None:",
            self.0.name, self.0.r#type
        )?;
        let doc = if let Some(default) = &self.0.default {
            if default == "..." {
                Cow::Borrowed(self.0.doc)
            } else {
                Cow::Owned(format!(
                    "{}\n```python\ndefault = {default}\n```",
                    self.0.doc
                ))
            }
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
