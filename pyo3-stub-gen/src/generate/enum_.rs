use crate::{generate::*, type_info::*};
use std::fmt;

/// Definition of a Python enum.
#[derive(Debug, Clone, PartialEq, getset::Getters)]
pub struct EnumDef {
    #[getset(get = "pub")]
    name: &'static str,
    #[getset(get = "pub")]
    doc: &'static str,
    #[getset(get = "pub")]
    variants: &'static [&'static str],
}

impl From<&PyEnumInfo> for EnumDef {
    fn from(info: &PyEnumInfo) -> Self {
        Self {
            name: info.pyclass_name,
            doc: info.doc,
            variants: info.variants,
        }
    }
}

impl fmt::Display for EnumDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "@final")?;
        writeln!(f, "class {}(Enum):", self.name)?;
        let indent = indent();
        let doc = self.doc.trim();
        if !doc.is_empty() {
            writeln!(f, r#"{indent}r""""#)?;
            for line in doc.lines() {
                writeln!(f, "{indent}{}", line)?;
            }
            writeln!(f, r#"{indent}""""#)?;
        }
        for variants in self.variants {
            writeln!(f, "{indent}{} = auto()", variants)?;
        }
        writeln!(f)?;
        Ok(())
    }
}
