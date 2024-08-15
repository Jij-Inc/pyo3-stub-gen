use crate::generate::*;
use itertools::Itertools;
use std::{
    any::TypeId,
    collections::{BTreeMap, BTreeSet},
    fmt,
};

/// Type info for a Python (sub-)module. This corresponds to a single `*.pyi` file.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Module {
    pub class: BTreeMap<TypeId, ClassDef>,
    pub enum_: BTreeMap<TypeId, EnumDef>,
    pub function: BTreeMap<&'static str, FunctionDef>,
    pub error: BTreeMap<&'static str, ErrorDef>,
    pub name: String,
    pub default_module_name: String,
    /// Direct submodules of this module.
    pub submodules: BTreeSet<String>,
}

impl Import for Module {
    fn import(&self) -> HashSet<ModuleRef> {
        let mut imports = HashSet::new();
        for class in self.class.values() {
            imports.extend(class.import());
        }
        for function in self.function.values() {
            imports.extend(function.import());
        }
        imports
    }
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "# This file is automatically generated by pyo3_stub_gen")?;
        writeln!(f, "# ruff: noqa: E501, F401")?;
        writeln!(f)?;
        for import in self.import().into_iter().sorted() {
            let name = import.get().unwrap_or(&self.default_module_name);
            if name != self.name {
                writeln!(f, "import {}", name)?;
            }
        }
        for submod in &self.submodules {
            writeln!(f, "from . import {}", submod)?;
        }
        if !self.enum_.is_empty() {
            writeln!(f, "from enum import Enum, auto")?;
        }
        writeln!(f)?;

        for class in self.class.values().sorted_by_key(|class| class.name) {
            write!(f, "{}", class)?;
        }
        for enum_ in self.enum_.values().sorted_by_key(|class| class.name) {
            write!(f, "{}", enum_)?;
        }
        for function in self.function.values() {
            write!(f, "{}", function)?;
        }
        for error in self.error.values() {
            writeln!(f, "{}", error)?;
        }
        Ok(())
    }
}
