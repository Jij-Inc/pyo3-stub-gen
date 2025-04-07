use crate::{generate::*, type_info::*};
use std::fmt;

/// Definition of `__new__` method.
#[derive(Debug, Clone, PartialEq)]
pub struct NewDef {
    pub args: Vec<Arg>,
    pub doc: &'static str,
}

impl Import for NewDef {
    fn import(&self) -> HashSet<ModuleRef> {
        let mut import = HashSet::new();
        for arg in &self.args {
            import.extend(arg.import().into_iter());
        }
        import
    }
}

impl From<&NewInfo> for NewDef {
    fn from(info: &NewInfo) -> Self {
        Self {
            args: info.args.iter().map(Arg::from).collect(),
            doc: info.doc,
        }
    }
}

impl fmt::Display for NewDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent = indent();
        write!(f, "{indent}def __new__(cls,")?;
        for (n, arg) in self.args.iter().enumerate() {
            write!(f, "{}", arg)?;
            if n != self.args.len() - 1 {
                write!(f, ", ")?;
            }
        }
        writeln!(f, "):")?;
        write!(f, "{}", DocPrinter(self.doc))?;
        writeln!(f, "{indent}{indent}...")?;
        writeln!(f)?;

        Ok(())
    }
}
