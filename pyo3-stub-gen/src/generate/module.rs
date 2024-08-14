use crate::generate::*;
use itertools::Itertools;
use std::{any::TypeId, collections::BTreeMap, fmt};

/// Type info for a Python (sub-)module. This corresponds to a single `*.pyi` file.
#[derive(Debug, Clone, PartialEq, Default, getset::Getters, getset::MutGetters)]
pub struct Module {
    #[getset(get = "pub", get_mut = "pub")]
    class: BTreeMap<TypeId, ClassDef>,
    #[getset(get = "pub", get_mut = "pub")]
    enum_: BTreeMap<TypeId, EnumDef>,
    #[getset(get = "pub", get_mut = "pub")]
    function: BTreeMap<&'static str, FunctionDef>,
    #[getset(get = "pub", get_mut = "pub")]
    error: BTreeMap<&'static str, ErrorDef>,
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "# This file is automatically generated by pyo3_stub_gen")?;
        writeln!(f, "# ruff: noqa: E501, F401")?;
        writeln!(f)?;
        writeln!(
            f,
            "from typing import final, Any, List, Dict, Sequence, Mapping"
        )?;
        writeln!(f, "from enum import Enum, auto")?;
        writeln!(f)?;

        for class in self.class.values().sorted_by_key(|class| class.name()) {
            write!(f, "{}", class)?;
        }
        for enum_ in self.enum_.values().sorted_by_key(|class| class.name()) {
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
