use crate::{generate::*, type_info::*, TypeInfo};
use std::fmt;

/// Definition of a Python function.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub name: &'static str,
    pub args: Vec<Arg>,
    pub r#return: TypeInfo,
    pub doc: &'static str,
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

impl Build for PyFunctionInfo {
    type DefType = FunctionDef;

    fn build(&self, current_module_name: &str) -> Self::DefType {
        Self::DefType {
            name: self.name,
            args: self.args.build(current_module_name),
            r#return: self.r#return.build(current_module_name),
            doc: self.doc,
        }
    }
}

impl fmt::Display for FunctionDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "def {}(", self.name)?;
        for (i, arg) in self.args.iter().enumerate() {
            write!(f, "{}", arg)?;
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
