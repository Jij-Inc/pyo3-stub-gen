use crate::{generate::*, type_info::*};
use std::fmt;

/// Definition of a Python enum.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub name: &'static str,
    pub doc: &'static str,
    pub variants: &'static [&'static str],
    pub methods: Vec<MethodDef>,
    pub members: Vec<MemberDef>,
}

impl From<&PyEnumInfo> for EnumDef {
    fn from(info: &PyEnumInfo) -> Self {
        Self {
            name: info.pyclass_name,
            doc: info.doc,
            variants: info.variants,
            methods: Vec::new(),
            members: Vec::new(),
        }
    }
}

impl fmt::Display for EnumDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "class {}(Enum):", self.name)?;
        let indent = indent();
        docstring::write_docstring(f, self.doc, indent)?;
        for variants in self.variants {
            writeln!(f, "{indent}{} = auto()", variants)?;
        }
        for member in &self.members {
            writeln!(f)?;
            member.fmt(f)?;
        }
        for methods in &self.methods {
            writeln!(f)?;
            methods.fmt(f)?;
        }
        writeln!(f)?;
        Ok(())
    }
}
