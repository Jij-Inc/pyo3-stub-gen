use crate::{generate::*, type_info::*, TypeInfo};
use std::fmt;

/// Definition of a Python class.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassDef {
    pub name: &'static str,
    pub doc: &'static str,
    pub members: Vec<MemberDef>,
    pub getters: Vec<MemberDef>,
    pub setters: Vec<MemberDef>,
    pub methods: Vec<MethodDef>,
    pub bases: Vec<TypeInfo>,
}

impl Import for ClassDef {
    fn import(&self) -> HashSet<ModuleRef> {
        let mut import = HashSet::new();
        for base in &self.bases {
            import.extend(base.import.clone());
        }
        for member in &self.members {
            import.extend(member.import());
        }
        for getter in &self.getters {
            import.extend(getter.import());
        }
        for setter in &self.setters {
            import.extend(setter.import());
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
            members: Vec::new(),
            setters: info.setters.iter().map(MemberDef::from).collect(),
            getters: info.getters.iter().map(MemberDef::from).collect(),
            methods: Vec::new(),
            bases: info.bases.iter().map(|f| f()).collect(),
        }
    }
}

impl fmt::Display for ClassDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bases = self
            .bases
            .iter()
            .map(|i| i.name.clone())
            .reduce(|acc, path| format!("{acc}, {path}"))
            .map(|bases| format!("({bases})"))
            .unwrap_or_default();
        writeln!(f, "class {}{}:", self.name, bases)?;
        let indent = indent();
        let doc = self.doc.trim();
        docstring::write_docstring(f, doc, indent)?;
        for member in &self.members {
            member.fmt(f)?;
        }
        for getter in &self.getters {
            GetterDisplay(getter).fmt(f)?;
        }
        for setter in &self.setters {
            SetterDisplay(setter).fmt(f)?;
        }
        for method in &self.methods {
            method.fmt(f)?;
        }
        if self.members.is_empty()
            && self.getters.is_empty()
            && self.setters.is_empty()
            && self.methods.is_empty()
        {
            writeln!(f, "{indent}...")?;
        }
        writeln!(f)?;
        Ok(())
    }
}
