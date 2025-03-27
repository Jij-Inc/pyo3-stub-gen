use crate::{generate::*, type_info::*};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct VariantDef {
    pub name: &'static str,
    pub doc: &'static str,
}

/// Definition of a Python enum.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub name: &'static str,
    pub doc: &'static str,
    pub variants: Vec<VariantDef>,
    pub methods: Vec<MethodDef>,
}

impl Import for EnumDef {
    fn import(&self) -> HashSet<ModuleRef> {
        let mut import = HashSet::new();
        for method in &self.methods {
            import.extend(method.import());
        }
        import
    }
}

impl From<&PyEnumInfo> for EnumDef {
    fn from(info: &PyEnumInfo) -> Self {
        Self {
            name: info.pyclass_name,
            doc: info.doc,
            variants: info
                .variants
                .iter()
                .map(|(variant, doc)| VariantDef { name: variant, doc })
                .collect(),
            methods: Vec::new(),
        }
    }
}

impl fmt::Display for EnumDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "class {}:", self.name)?;
        let indent = indent();
        let doc = self.doc.trim();
        if !doc.is_empty() {
            writeln!(f, r#"{indent}r""""#)?;
            for line in doc.lines() {
                writeln!(f, "{indent}{}", line)?;
            }
            writeln!(f, r#"{indent}""""#)?;
        }
        for variant in &self.variants {
            writeln!(f, "{indent}{}: {}", variant.name, self.name)?;
            if !variant.doc.is_empty() {
                writeln!(f, r#"{indent}r""""#)?;
                for line in variant.doc.lines() {
                    if line.is_empty() {
                        writeln!(f)?;
                    } else {
                        writeln!(f, "{indent}{}", line)?;
                    }
                }
                writeln!(f, r#"{indent}""""#)?;
            }
        }
        writeln!(f)?;

        for method in &self.methods {
            method.fmt(f)?;
        }

        Ok(())
    }
}
