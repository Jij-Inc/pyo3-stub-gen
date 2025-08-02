use crate::{generate::*, type_info::*, TypeInfo};
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
        }
    }
}

impl fmt::Display for FunctionDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Add deprecated decorator if present
        if let Some(deprecated) = &self.deprecated {
            write!(f, "@typing_extensions.deprecated(")?;
            if let Some(since) = deprecated.since {
                write!(f, "\"since={since}")?;
                if let Some(note) = deprecated.note {
                    write!(f, ", {note}")?;
                }
                write!(f, "\"")?;
            } else if let Some(note) = deprecated.note {
                write!(f, "\"{note}\"")?;
            } else {
                write!(f, "\"\"")?;
            }
            writeln!(f, ")")?;
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
