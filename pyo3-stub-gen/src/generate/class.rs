use indexmap::IndexMap;

use crate::generate::variant_methods::get_variant_methods;
use crate::{generate::*, type_info::*, TypeInfo};
use std::{fmt, vec};

/// Definition of a Python class.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassDef {
    pub name: &'static str,
    pub doc: &'static str,
    pub attrs: Vec<MemberDef>,
    pub getter_setters: IndexMap<String, (Option<MemberDef>, Option<MemberDef>)>,
    pub methods: IndexMap<String, Vec<MethodDef>>,
    pub bases: Vec<TypeInfo>,
    pub classes: Vec<ClassDef>,
    pub match_args: Option<Vec<String>>,
    pub subclass: bool,
}

impl Import for ClassDef {
    fn import(&self) -> HashSet<ImportRef> {
        let mut import = HashSet::new();
        if !self.subclass {
            // for @typing.final
            import.insert("typing".into());
        }
        for base in &self.bases {
            import.extend(base.import.clone());
        }
        for attr in &self.attrs {
            import.extend(attr.import());
        }
        for (getter, setter) in self.getter_setters.values() {
            if let Some(getter) = getter {
                import.extend(getter.import());
            }
            if let Some(setter) = setter {
                import.extend(setter.import());
            }
        }
        for method in self.methods.values() {
            if method.len() > 1 {
                // for @typing.overload
                import.insert("typing".into());
            }
            for method in method {
                import.extend(method.import());
            }
        }
        for class in &self.classes {
            import.extend(class.import());
        }
        import
    }
}

impl From<&PyComplexEnumInfo> for ClassDef {
    fn from(info: &PyComplexEnumInfo) -> Self {
        // Since there are multiple `#[pymethods]` for a single class, we need to merge them.
        // This is only an initializer. See `StubInfo::gather` for the actual merging.

        let enum_info = Self {
            name: info.pyclass_name,
            doc: info.doc,
            getter_setters: IndexMap::new(),
            methods: IndexMap::new(),
            classes: info
                .variants
                .iter()
                .map(|v| ClassDef::from_variant(info, v))
                .collect(),
            bases: Vec::new(),
            match_args: None,
            attrs: Vec::new(),
            subclass: true, // Complex enums can be subclassed by their variants
        };

        enum_info
    }
}

impl ClassDef {
    fn from_variant(enum_info: &PyComplexEnumInfo, info: &VariantInfo) -> Self {
        let methods = get_variant_methods(enum_info, info);

        Self {
            name: info.pyclass_name,
            doc: info.doc,
            getter_setters: info
                .fields
                .iter()
                .map(|info| (info.name.to_string(), (Some(MemberDef::from(info)), None)))
                .collect(),
            methods,
            classes: Vec::new(),
            bases: vec![TypeInfo::unqualified(enum_info.pyclass_name)],
            match_args: Some(info.fields.iter().map(|f| f.name.to_string()).collect()),
            attrs: Vec::new(),
            subclass: false,
        }
    }
}

impl From<&PyClassInfo> for ClassDef {
    fn from(info: &PyClassInfo) -> Self {
        // Since there are multiple `#[pymethods]` for a single class, we need to merge them.
        // This is only an initializer. See `StubInfo::gather` for the actual merging.
        let mut getter_setters: IndexMap<String, (Option<MemberDef>, Option<MemberDef>)> = info
            .getters
            .iter()
            .map(|info| (info.name.to_string(), (Some(MemberDef::from(info)), None)))
            .collect();
        for setter in info.setters {
            getter_setters.entry(setter.name.to_string()).or_default().1 = Some(MemberDef {
                name: setter.name,
                r#type: (setter.r#type)(),
                doc: setter.doc,
                default: setter.default.map(|f| f()),
                deprecated: setter.deprecated.clone(),
            });
        }
        let mut new = Self {
            name: info.pyclass_name,
            doc: info.doc,
            attrs: Vec::new(),
            getter_setters,
            methods: Default::default(),
            classes: Vec::new(),
            bases: info.bases.iter().map(|f| f()).collect(),
            match_args: None,
            subclass: info.subclass,
        };
        if info.has_eq {
            new.add_eq_method();
        }
        if info.has_ord {
            new.add_ord_methods();
        }
        if info.has_hash {
            new.add_hash_method();
        }
        if info.has_str {
            new.add_str_method();
        }
        new
    }
}
impl ClassDef {
    fn add_eq_method(&mut self) {
        let method = MethodDef {
            name: "__eq__",
            args: vec![Arg {
                name: "other",
                r#type: TypeInfo::builtin("object"),
                signature: None,
            }],
            r#return: TypeInfo::builtin("bool"),
            doc: "",
            r#type: MethodType::Instance,
            is_async: false,
            deprecated: None,
            type_ignored: None,
        };
        self.methods
            .entry("__eq__".to_string())
            .or_default()
            .push(method);
    }

    fn add_ord_methods(&mut self) {
        let ord_methods = ["__lt__", "__le__", "__gt__", "__ge__"];

        for name in &ord_methods {
            let method = MethodDef {
                name,
                args: vec![Arg {
                    name: "other",
                    r#type: TypeInfo::builtin("object"),
                    signature: None,
                }],
                r#return: TypeInfo::builtin("bool"),
                doc: "",
                r#type: MethodType::Instance,
                is_async: false,
                deprecated: None,
                type_ignored: None,
            };
            self.methods
                .entry(name.to_string())
                .or_default()
                .push(method);
        }
    }

    fn add_hash_method(&mut self) {
        let method = MethodDef {
            name: "__hash__",
            args: vec![],
            r#return: TypeInfo::builtin("int"),
            doc: "",
            r#type: MethodType::Instance,
            is_async: false,
            deprecated: None,
            type_ignored: None,
        };
        self.methods
            .entry("__hash__".to_string())
            .or_default()
            .push(method);
    }

    fn add_str_method(&mut self) {
        let method = MethodDef {
            name: "__str__",
            args: vec![],
            r#return: TypeInfo::builtin("str"),
            doc: "",
            r#type: MethodType::Instance,
            is_async: false,
            deprecated: None,
            type_ignored: None,
        };
        self.methods
            .entry("__str__".to_string())
            .or_default()
            .push(method);
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
        if !self.subclass {
            writeln!(f, "@typing.final")?;
        }
        writeln!(f, "class {}{}:", self.name, bases)?;
        let indent = indent();
        let doc = self.doc.trim();
        docstring::write_docstring(f, doc, indent)?;

        if let Some(match_args) = &self.match_args {
            if match_args.is_empty() {
                writeln!(f, "{indent}__match_args__ = ()")?;
            } else {
                let match_args_txt = match_args
                    .iter()
                    .map(|a| format!(r##""{a}""##))
                    .collect::<Vec<_>>()
                    .join(", ");
                writeln!(f, "{indent}__match_args__ = ({match_args_txt},)")?;
            }
        }
        for attr in &self.attrs {
            attr.fmt(f)?;
        }
        for (getter, setter) in self.getter_setters.values() {
            if let Some(getter) = getter {
                GetterDisplay(getter).fmt(f)?;
            }
            if let Some(setter) = setter {
                SetterDisplay(setter).fmt(f)?;
            }
        }
        for methods in self.methods.values() {
            let overloaded = methods.len() > 1;
            for method in methods {
                if overloaded {
                    writeln!(f, "{indent}@typing.overload")?;
                }
                method.fmt(f)?;
            }
        }
        for class in &self.classes {
            let emit = format!("{class}");
            for line in emit.lines() {
                writeln!(f, "{indent}{line}")?;
            }
        }
        if self.attrs.is_empty() && self.getter_setters.is_empty() && self.methods.is_empty() {
            writeln!(f, "{indent}...")?;
        }
        writeln!(f)?;
        Ok(())
    }
}
