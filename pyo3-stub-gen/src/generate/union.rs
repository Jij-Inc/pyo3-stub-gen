use crate::{generate::*, type_info::*, TypeInfo};
use std::fmt;

/// Definition of a Python enum.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeUnionDef {
    pub name: &'static str,
    pub doc: &'static str,
    pub variants: Vec<TypeInfo>,
}

impl From<&TypeUnionEnumInfo> for TypeUnionDef {
    fn from(info: &TypeUnionEnumInfo) -> Self {
        Self {
            name: info.pyclass_name,
            doc: info.doc,
            variants: info.variants.iter().map(|t| t()).collect(),
        }
    }
}

impl fmt::Display for TypeUnionDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} = ", self.name)?;
        let indent = indent();

        for (i, variant_name) in self.variants.iter().enumerate() {
            if i != 0 {
                write!(f, " | ")?;
            }
            write!(f, "{variant_name}")?;
        }
        writeln!(f, "\n")?;

        // Docstrings don't really seem to be supported, but pyright uses mult-line strings below
        // definition:
        // https://discuss.python.org/t/docstrings-for-new-type-aliases-as-defined-in-pep-695/39816
        docstring::write_docstring(f, self.doc, indent)?;

        Ok(())
    }
}
