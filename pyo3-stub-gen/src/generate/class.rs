use crate::{generate::*, type_info::*};
use std::fmt;

/// Definition of a Python class.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassDef {
    pub name: &'static str,
    pub doc: &'static str,
    pub members: Vec<MemberDef>,
    pub methods: Vec<MethodDef>,
    pub bases: &'static [(Option<&'static str>, &'static str)],
}

impl Import for ClassDef {
    fn import(&self) -> HashSet<ModuleRef> {
        let mut import = HashSet::new();
        for member in &self.members {
            import.extend(member.import());
        }
        for method in &self.methods {
            import.extend(method.import());
        }
        import
    }
}

impl From<&PyClassInfo> for ClassDef {
    fn from(info: &PyClassInfo) -> Self {
        // Since there are multiple `#[pymethods]` for a single class, we need to merge them.
        // This is only an initializer. See `StubInfo::gather` for the actual merging.
        Self {
            name: info.pyclass_name,
            doc: info.doc,
            members: info.members.iter().map(MemberDef::from).collect(),
            methods: Vec::new(),
            bases: info.bases,
        }
    }
}

impl fmt::Display for ClassDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bases = self
            .bases
            .iter()
            .map(|(m, n)| {
                m.map(|m| format!("{m}.{n}"))
                    .unwrap_or_else(|| n.to_string())
            })
            .reduce(|acc, path| format!("{acc}, {path}"))
            .map(|bases| format!("({bases})"))
            .unwrap_or_default();
        writeln!(f, "class {}{}:", self.name, bases)?;
        let indent = indent();
        let doc = self.doc.trim();
        if !doc.is_empty() {
            writeln!(f, r#"{indent}r""""#)?;
            for line in doc.lines() {
                writeln!(f, "{indent}{}", line)?;
            }
            writeln!(f, r#"{indent}""""#)?;
        }
        for member in &self.members {
            member.fmt(f)?;
        }
        for method in &self.methods {
            method.fmt(f)?;
        }
        if self.members.is_empty() && self.methods.is_empty() {
            writeln!(f, "{indent}...")?;
        }
        writeln!(f)?;
        Ok(())
    }
}
