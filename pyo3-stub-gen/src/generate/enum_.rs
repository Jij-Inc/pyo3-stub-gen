use crate::{generate::*, type_info::*};
use std::fmt;

/// Definition of a Python enum.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub name: &'static str,
    pub doc: &'static str,
    pub variants: &'static [(&'static str, &'static str)],
    pub methods: Vec<MethodDef>,
    pub attrs: Vec<MemberDef>,
    pub getters: Vec<MemberDef>,
    pub setters: Vec<MemberDef>,
}

impl From<&PyEnumInfo> for EnumDef {
    fn from(info: &PyEnumInfo) -> Self {
        Self {
            name: info.pyclass_name,
            doc: info.doc,
            variants: info.variants,
            methods: Vec::new(),
            attrs: Vec::new(),
            getters: Vec::new(),
            setters: Vec::new(),
        }
    }
}

impl fmt::Display for EnumDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "class {}(Enum):", self.name)?;
        let indent = indent();
        docstring::write_docstring(f, self.doc, indent)?;
        for (variant, variant_doc) in self.variants {
            writeln!(f, "{indent}{} = ...", variant)?;
            docstring::write_docstring(f, variant_doc, indent)?;
        }
        if !(self.attrs.is_empty()
            && self.getters.is_empty()
            && self.getters.is_empty()
            && self.methods.is_empty())
        {
            writeln!(f)?;
            for attr in &self.attrs {
                attr.fmt(f)?;
            }
            for getter in &self.getters {
                GetterDisplay(getter).fmt(f)?;
            }
            for setter in &self.setters {
                SetterDisplay(setter).fmt(f)?;
            }
            for methods in &self.methods {
                methods.fmt(f)?;
            }
        }
        writeln!(f)?;
        Ok(())
    }
}
