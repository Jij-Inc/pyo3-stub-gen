use crate::{generate::*, type_info::*, TypeInfo};
use std::fmt;

/// Definition of a Python class.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassDef {
    pub name: &'static str,
    pub doc: &'static str,
    pub members: Vec<MemberDef>,
    pub methods: Vec<MethodDef>,
    pub bases: Vec<TypeInfo>,
    pub classes: Vec<ClassDef>
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
        for method in &self.methods {
            import.extend(method.import());
        }
        for class in &self.classes {
            import.extend(class.import());
        }
        import
    }
}

impl From<&PyRichEnumInfo> for ClassDef {
    fn from(info: &PyRichEnumInfo) -> Self {
        // Since there are multiple `#[pymethods]` for a single class, we need to merge them.
        // This is only an initializer. See `StubInfo::gather` for the actual merging.
        Self {
            name: info.pyclass_name,
            doc: info.doc,
            members: Vec::new(),
            methods: Vec::new(),
            classes: info.variants.iter().map(ClassDef::from).collect(),
            bases: Vec::new(),
        }
    }
}

impl From<&VariantInfo> for ClassDef {
    fn from(info: &VariantInfo) -> Self {
        Self {
            name: info.pyclass_name,
            doc: info.doc,
            members: info.fields.iter().map(MemberDef::from).collect(),
            methods: Vec::new(),
            classes: Vec::new(),
            bases: Vec::new(),
        }
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
            classes: Vec::new(),
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
        for method in &self.methods {
            method.fmt(f)?;
        }
        for class in &self.classes {
            let emit = format!("{}", class);
            for line in emit.lines() {
                writeln!(f, "{}{}", indent, line)?;
            }
        }
        if self.members.is_empty() && self.methods.is_empty() && self.classes.is_empty() {
            writeln!(f, "{indent}...")?;
        }
        writeln!(f)?;
        Ok(())
    }
}
