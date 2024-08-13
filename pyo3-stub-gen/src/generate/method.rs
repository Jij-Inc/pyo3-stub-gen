use crate::{generate::*, type_info::*};
use pyo3::inspect::types::TypeInfo;
use std::fmt;

/// Definition of a class method.
#[derive(Debug, Clone, PartialEq)]
pub struct MethodDef {
    pub name: &'static str,
    pub args: Vec<Arg>,
    pub signature: Option<&'static str>,
    pub r#return: TypeInfo,
    pub doc: &'static str,
    pub is_static: bool,
    pub is_class: bool,
}

impl From<&MethodInfo> for MethodDef {
    fn from(info: &MethodInfo) -> Self {
        Self {
            name: info.name,
            args: info.args.iter().map(Arg::from).collect(),
            signature: info.signature,
            r#return: (info.r#return)().into(),
            doc: info.doc,
            is_static: info.is_static,
            is_class: info.is_class,
        }
    }
}

impl fmt::Display for MethodDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent = indent();
        let mut needs_comma = false;
        if self.is_static {
            writeln!(f, "{indent}@staticmethod")?;
            write!(f, "{indent}def {}(", self.name)?;
        } else if self.is_class {
            writeln!(f, "{indent}@classmethod")?;
            write!(f, "{indent}def {}(cls", self.name)?;
            needs_comma = true;
        } else {
            write!(f, "{indent}def {}(self", self.name)?;
            needs_comma = true;
        }
        if let Some(signature) = self.signature {
            if needs_comma {
                write!(f, ", ")?;
            }
            write!(f, "{}", signature)?;
        } else {
            for arg in &self.args {
                if needs_comma {
                    write!(f, ", ")?;
                }
                write!(f, "{}", arg)?;
                needs_comma = true;
            }
        }
        writeln!(f, ") -> {}:", self.r#return)?;

        let doc = self.doc;
        if !doc.is_empty() {
            writeln!(f, r#"{indent}{indent}r""""#)?;
            for line in doc.lines() {
                writeln!(f, "{indent}{indent}{}", line)?;
            }
            writeln!(f, r#"{indent}{indent}""""#)?;
        }
        writeln!(f, "{indent}{indent}...")?;
        writeln!(f)?;
        Ok(())
    }
}
