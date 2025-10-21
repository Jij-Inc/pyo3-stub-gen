use crate::stub_type::ImportRef;
use crate::{generate::*, rule_name::RuleName, type_info::*, TypeInfo};
use itertools::Itertools;
use std::fmt;

/// Definition of a Python function.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub name: &'static str,
    pub parameters: Parameters,
    pub r#return: TypeInfo,
    pub doc: &'static str,
    pub is_async: bool,
    pub deprecated: Option<DeprecatedInfo>,
    pub type_ignored: Option<IgnoreTarget>,
}

impl Import for FunctionDef {
    fn import(&self) -> HashSet<ImportRef> {
        let mut import = self.r#return.import.clone();
        import.extend(self.parameters.import());
        // Add typing_extensions import if deprecated
        if self.deprecated.is_some() {
            import.insert("typing_extensions".into());
        }
        import
    }
}

impl From<&PyFunctionInfo> for FunctionDef {
    fn from(info: &PyFunctionInfo) -> Self {
        Self {
            name: info.name,
            parameters: Parameters::from_infos(info.parameters),
            r#return: (info.r#return)(),
            doc: info.doc,
            is_async: info.is_async,
            deprecated: info.deprecated.clone(),
            type_ignored: info.type_ignored,
        }
    }
}

impl fmt::Display for FunctionDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Add deprecated decorator if present
        if let Some(deprecated) = &self.deprecated {
            writeln!(f, "{deprecated}")?;
        }

        let async_ = if self.is_async { "async " } else { "" };
        write!(
            f,
            "{async_}def {}({}) -> {}:",
            self.name, self.parameters, self.r#return
        )?;

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
                    Some(format!("  # type: ignore[{rules_str}]"))
                }
            }
        } else {
            None
        };

        let doc = self.doc;
        if !doc.is_empty() {
            // Add type: ignore comment for functions with docstrings
            if let Some(comment) = &type_ignore_comment {
                write!(f, "{comment}")?;
            }
            writeln!(f)?;
            docstring::write_docstring(f, self.doc, indent())?;
        } else {
            write!(f, " ...")?;
            // Add type: ignore comment for functions without docstrings
            if let Some(comment) = &type_ignore_comment {
                write!(f, "{comment}")?;
            }
            writeln!(f)?;
        }
        writeln!(f)?;
        Ok(())
    }
}
