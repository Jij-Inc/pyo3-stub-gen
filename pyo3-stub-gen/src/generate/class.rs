use crate::generate::variant_methods::get_variant_methods;
use crate::{generate::*, type_info::*, TypeInfo};
use std::{fmt, vec};

/// Definition of a Python class.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassDef {
    pub name: &'static str,
    pub doc: &'static str,
    pub attrs: Vec<MemberDef>,
    pub getters: Vec<MemberDef>,
    pub setters: Vec<MemberDef>,
    pub methods: Vec<MethodDef>,
    pub bases: Vec<TypeInfo>,
    pub classes: Vec<ClassDef>,
    pub match_args: Option<Vec<String>>,
}

impl Import for ClassDef {
    fn import(&self) -> HashSet<ModuleRef> {
        let mut import = HashSet::new();
        for base in &self.bases {
            import.extend(base.import.clone());
        }
        for attr in &self.attrs {
            import.extend(attr.import());
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

        let enum_info = Self {
            name: info.pyclass_name,
            doc: info.doc,
            getters: Vec::new(),
            setters: Vec::new(),
            methods: Vec::new(),
            classes: info
                .variants
                .iter()
                .map(|v| ClassDef::from_variant(info, v))
                .collect(),
            bases: Vec::new(),
            match_args: None,
            attrs: Vec::new(),
        };

        enum_info
    }
}

impl ClassDef {
    fn from_variant(enum_info: &PyRichEnumInfo, info: &VariantInfo) -> Self {
        let methods = get_variant_methods(enum_info, info);

        Self {
            name: info.pyclass_name,
            doc: info.doc,
            getters: info.fields.iter().map(MemberDef::from).collect(),
            setters: Vec::new(),
            methods,
            classes: Vec::new(),
            bases: vec![TypeInfo::unqualified(enum_info.pyclass_name)],
            match_args: Some(info.fields.iter().map(|f| f.name.to_string()).collect()),
            attrs: Vec::new(),
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
            attrs: Vec::new(),
            setters: info.setters.iter().map(MemberDef::from).collect(),
            getters: info.getters.iter().map(MemberDef::from).collect(),
            methods: Vec::new(),
            classes: Vec::new(),
            bases: info.bases.iter().map(|f| f()).collect(),
            match_args: None,
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

        if let Some(match_args) = &self.match_args {
            let match_args_txt = if match_args.is_empty() {
                "()".to_string()
            } else {
                match_args
                    .iter()
                    .map(|a| format!(r##""{a}""##))
                    .collect::<Vec<_>>()
                    .join(", ")
            };

            writeln!(f, "{indent}__match_args__ = ({match_args_txt},)")?;
        }
        for attr in &self.attrs {
            attr.fmt(f)?;
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
        for class in &self.classes {
            let emit = format!("{class}");
            for line in emit.lines() {
                writeln!(f, "{indent}{line}")?;
            }
        }
        if self.attrs.is_empty()
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
