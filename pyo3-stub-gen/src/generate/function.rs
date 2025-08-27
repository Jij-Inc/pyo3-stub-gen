use crate::{generate::*, rule_name::RuleName, type_info::*, TypeInfo};
use itertools::Itertools;
use std::fmt;

/// Definition of a Python function.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub name: &'static str,
    pub args: Vec<Arg>,
    pub r#return: TypeInfo,
    pub doc: &'static str,
    pub is_async: bool,
    pub deprecated: Option<DeprecatedInfo>,
    pub type_ignored: Option<IgnoreTarget>,
}

impl Import for FunctionDef {
    fn import(&self) -> HashSet<ModuleRef> {
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

impl From<&PyFunctionInfo> for FunctionDef {
    fn from(info: &PyFunctionInfo) -> Self {
        Self {
            name: info.name,
            args: info.args.iter().map(Arg::from).collect(),
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
        write!(f, "{async_}def {}(", self.name)?;
        for (i, arg) in self.args.iter().enumerate() {
            write!(f, "{arg}")?;
            if i != self.args.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, ") -> {}:", self.r#return)?;

        // Add type: ignore comment if needed
        if let Some(target) = &self.type_ignored {
            match target {
                IgnoreTarget::All => {
                    write!(f, "  # type: ignore")?;
                }
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
                    write!(f, "  # type: ignore[{}]", rules_str)?;
                }
            }
        }

        let doc = self.doc;
        if !doc.is_empty() {
            writeln!(f)?;
            docstring::write_docstring(f, self.doc, indent())?;
        } else {
            writeln!(f, " ...")?;
        }
        writeln!(f)?;
        Ok(())
    }
}
