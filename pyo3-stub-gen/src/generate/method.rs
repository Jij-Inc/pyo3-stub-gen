use crate::{generate::*, rule_name::RuleName, type_info::*, TypeInfo};
use itertools::Itertools;
use std::{collections::HashSet, fmt};

pub use crate::type_info::MethodType;

/// Definition of a class method.
#[derive(Debug, Clone, PartialEq)]
pub struct MethodDef {
    pub name: &'static str,
    pub args: Vec<Arg>,
    pub r#return: TypeInfo,
    pub doc: &'static str,
    pub r#type: MethodType,
    pub is_async: bool,
    pub deprecated: Option<DeprecatedInfo>,
    pub type_ignored: Option<IgnoreTarget>,
}

impl Import for MethodDef {
    fn import(&self) -> HashSet<ImportRef> {
        let mut import = self.r#return.import.clone();
        for arg in &self.args {
            import.extend(arg.import().into_iter());
        }
        // Add typing_extensions import if deprecated
        if self.deprecated.is_some() {
            import.insert("typing_extensions".into());
        }
        import
    }
}

impl From<&MethodInfo> for MethodDef {
    fn from(info: &MethodInfo) -> Self {
        Self {
            name: info.name,
            args: info.args.iter().map(Arg::from).collect(),
            r#return: (info.r#return)(),
            doc: info.doc,
            r#type: info.r#type,
            is_async: info.is_async,
            deprecated: info.deprecated.clone(),
            type_ignored: info.type_ignored,
        }
    }
}

impl fmt::Display for MethodDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent = indent();
        let mut needs_comma = false;
        let async_ = if self.is_async { "async " } else { "" };

        // Add deprecated decorator if present
        if let Some(deprecated) = &self.deprecated {
            writeln!(f, "{indent}{deprecated}")?;
        }

        match self.r#type {
            MethodType::Static => {
                writeln!(f, "{indent}@staticmethod")?;
                write!(f, "{indent}{async_}def {}(", self.name)?;
            }
            MethodType::Class | MethodType::New => {
                if self.r#type == MethodType::Class {
                    // new is a classmethod without the decorator
                    writeln!(f, "{indent}@classmethod")?;
                }
                write!(f, "{indent}{async_}def {}(cls", self.name)?;
                needs_comma = true;
            }
            MethodType::Instance => {
                write!(f, "{indent}{async_}def {}(self", self.name)?;
                needs_comma = true;
            }
        }
        for arg in &self.args {
            if needs_comma {
                write!(f, ", ")?;
            }
            write!(f, "{arg}")?;
            needs_comma = true;
        }
        write!(f, ") -> {}:", self.r#return)?;

        // Calculate type: ignore comment once
        let type_ignore_comment = if let Some(target) = &self.type_ignored {
            match target {
                IgnoreTarget::All => Some("  # type: ignore".to_string()),
                IgnoreTarget::Specified(rules) => {
                    let rules_str = rules
                        .iter()
                        .map(|r| {
                            let result = r.parse::<RuleName>().unwrap();
                            if let RuleName::Custom(custom) = &result {
                                log::warn!("Unknown custom rule name '{custom}' used in type ignore. Ensure this is intended.");
                            }
                            result
                        })
                        .join(",");
                    Some(format!("  # type: ignore[{}]", rules_str))
                }
            }
        } else {
            None
        };

        let doc = self.doc;
        if !doc.is_empty() {
            // Add type: ignore comment for methods with docstrings
            if let Some(comment) = &type_ignore_comment {
                write!(f, "{}", comment)?;
            }
            writeln!(f)?;
            let double_indent = format!("{indent}{indent}");
            docstring::write_docstring(f, self.doc, &double_indent)?;
        } else {
            write!(f, " ...")?;
            // Add type: ignore comment for methods without docstrings
            if let Some(comment) = &type_ignore_comment {
                write!(f, "{}", comment)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
