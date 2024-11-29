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
}

impl Import for FunctionDef {
    fn import(&self) -> HashSet<ModuleRef> {
        let mut import = self.r#return.import.clone();
        for arg in &self.args {
            import.extend(arg.import().into_iter());
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
        }
    }
}

impl fmt::Display for FunctionDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
