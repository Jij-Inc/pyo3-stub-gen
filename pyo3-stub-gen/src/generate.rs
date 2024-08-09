//! Generate Python typing stub file a.k.a. `*.pyi` file.

use crate::{pyproject::PyProject, type_info::*};

use anyhow::{anyhow, bail, Result};
use itertools::Itertools;
use pyo3::inspect::types::TypeInfo;
use std::{
    any::TypeId,
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    io::Write,
    path::*,
};

fn indent() -> &'static str {
    "    "
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arg {
    pub name: &'static str,
    pub r#type: TypeInfo,
}

impl From<&ArgInfo> for Arg {
    fn from(info: &ArgInfo) -> Self {
        Self {
            name: info.name,
            r#type: (info.r#type)(),
        }
    }
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.name, self.r#type)
    }
}

/// Wrapper of [TypeInfo] to implement [fmt::Display] which insert `->` before return type for non-`NoReturn` type.
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnTypeInfo {
    pub r#type: TypeInfo,
}

impl fmt::Display for ReturnTypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !matches!(self.r#type, TypeInfo::NoReturn) {
            write!(f, " -> {}", self.r#type)?;
        }
        Ok(())
    }
}

impl From<TypeInfo> for ReturnTypeInfo {
    fn from(r#type: TypeInfo) -> Self {
        Self { r#type }
    }
}

/// Definition of a class method.
#[derive(Debug, Clone, PartialEq)]
pub struct MethodDef {
    pub name: &'static str,
    pub args: Vec<Arg>,
    pub signature: Option<&'static str>,
    pub r#return: ReturnTypeInfo,
    pub doc: &'static str,
    pub is_static: bool,
    pub is_class: bool,
}

impl From<&MethodInfo> for MethodDef {
    fn from(info: &MethodInfo) -> Self {
        Self {
            name: info.name,
            args: info.args.iter().map(Arg::from).collect(),
            signature: info.signature,
            r#return: (info.r#return)().into(),
            doc: info.doc,
            is_static: info.is_static,
            is_class: info.is_class,
        }
    }
}

impl fmt::Display for MethodDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent = indent();
        let mut needs_comma = false;
        if self.is_static {
            writeln!(f, "{indent}@staticmethod")?;
            write!(f, "{indent}def {}(", self.name)?;
        } else if self.is_class {
            writeln!(f, "{indent}@classmethod")?;
            write!(f, "{indent}def {}(cls", self.name)?;
            needs_comma = true;
        } else {
            write!(f, "{indent}def {}(self", self.name)?;
            needs_comma = true;
        }
        if let Some(signature) = self.signature {
            if needs_comma {
                write!(f, ", ")?;
            }
            write!(f, "{}", signature)?;
        } else {
            for arg in &self.args {
                if needs_comma {
                    write!(f, ", ")?;
                }
                write!(f, "{}", arg)?;
                needs_comma = true;
            }
        }
        writeln!(f, "){}:", self.r#return)?;

        let doc = self.doc;
        if !doc.is_empty() {
            writeln!(f, r#"{indent}{indent}r""""#)?;
            for line in doc.lines() {
                writeln!(f, "{indent}{indent}{}", line)?;
            }
            writeln!(f, r#"{indent}{indent}""""#)?;
        }
        writeln!(f, "{indent}{indent}...")?;
        writeln!(f)?;
        Ok(())
    }
}

/// Definition of a class member.
#[derive(Debug, Clone, PartialEq)]
pub struct MemberDef {
    pub name: &'static str,
    pub r#type: TypeInfo,
}

impl From<&MemberInfo> for MemberDef {
    fn from(info: &MemberInfo) -> Self {
        Self {
            name: info.name,
            r#type: (info.r#type)(),
        }
    }
}

impl fmt::Display for MemberDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent = indent();
        writeln!(f, "{indent}{}: {}", self.name, self.r#type)
    }
}

/// Definition of `__new__` method.
#[derive(Debug, Clone, PartialEq)]
pub struct NewDef {
    pub args: Vec<Arg>,
    pub signature: Option<&'static str>,
}

impl From<&NewInfo> for NewDef {
    fn from(info: &NewInfo) -> Self {
        Self {
            args: info.args.iter().map(Arg::from).collect(),
            signature: info.signature,
        }
    }
}

impl fmt::Display for NewDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent = indent();
        write!(f, "{indent}def __new__(cls,")?;
        if let Some(signature) = self.signature {
            let joined = signature.replace('\n', " ");
            write!(f, "{}", joined)?;
        } else {
            for (n, arg) in self.args.iter().enumerate() {
                write!(f, "{}", arg)?;
                if n != self.args.len() - 1 {
                    write!(f, ", ")?;
                }
            }
        }
        writeln!(f, "): ...")?;
        Ok(())
    }
}

/// Definition of a Python class.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassDef {
    pub name: &'static str,
    pub doc: &'static str,
    pub new: Option<NewDef>,
    pub members: Vec<MemberDef>,
    pub methods: Vec<MethodDef>,
}

impl From<&PyClassInfo> for ClassDef {
    fn from(info: &PyClassInfo) -> Self {
        // Since there are multiple `#[pymethods]` for a single class, we need to merge them.
        // This is only an initializer. See `StubInfo::gather` for the actual merging.
        Self {
            name: info.pyclass_name,
            new: None,
            doc: info.doc,
            members: info.members.iter().map(MemberDef::from).collect(),
            methods: Vec::new(),
        }
    }
}

impl fmt::Display for ClassDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "@final")?;
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
        for member in &self.members {
            member.fmt(f)?;
        }
        if let Some(new) = &self.new {
            new.fmt(f)?;
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

/// Definition of a Python enum.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub name: &'static str,
    pub doc: &'static str,
    pub variants: &'static [&'static str],
}

impl From<&PyEnumInfo> for EnumDef {
    fn from(info: &PyEnumInfo) -> Self {
        Self {
            name: info.pyclass_name,
            doc: info.doc,
            variants: info.variants,
        }
    }
}

impl fmt::Display for EnumDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "@final")?;
        writeln!(f, "class {}(Enum):", self.name)?;
        let indent = indent();
        let doc = self.doc.trim();
        if !doc.is_empty() {
            writeln!(f, r#"{indent}r""""#)?;
            for line in doc.lines() {
                writeln!(f, "{indent}{}", line)?;
            }
            writeln!(f, r#"{indent}""""#)?;
        }
        for variants in self.variants {
            writeln!(f, "{indent}{} = auto()", variants)?;
        }
        writeln!(f)?;
        Ok(())
    }
}

/// Definition of a Python function.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub name: &'static str,
    pub args: Vec<Arg>,
    pub r#return: ReturnTypeInfo,
    pub signature: Option<&'static str>,
    pub doc: &'static str,
}

impl From<&PyFunctionInfo> for FunctionDef {
    fn from(info: &PyFunctionInfo) -> Self {
        Self {
            name: info.name,
            args: info.args.iter().map(Arg::from).collect(),
            r#return: (info.r#return)().into(),
            doc: info.doc,
            signature: info.signature,
        }
    }
}

impl fmt::Display for FunctionDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "def {}(", self.name)?;
        if let Some(sig) = self.signature {
            write!(f, "{}", sig)?;
        } else {
            for (i, arg) in self.args.iter().enumerate() {
                write!(f, "{}", arg)?;
                if i != self.args.len() - 1 {
                    write!(f, ",")?;
                }
            }
        }
        writeln!(f, "){}:", self.r#return)?;

        let doc = self.doc;
        let indent = indent();
        if !doc.is_empty() {
            writeln!(f, r#"{indent}r""""#)?;
            for line in doc.lines() {
                writeln!(f, "{indent}{}", line)?;
            }
            writeln!(f, r#"{indent}""""#)?;
        }
        writeln!(f, "{indent}...")?;
        writeln!(f)?;
        Ok(())
    }
}

/// Type info for a Python (sub-)module. This corresponds to a single `*.pyi` file.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Module {
    pub class: BTreeMap<TypeId, ClassDef>,
    pub enum_: BTreeMap<TypeId, EnumDef>,
    pub function: BTreeMap<&'static str, FunctionDef>,
    pub error: BTreeSet<&'static str>,
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

        for class in self.class.values().sorted_by_key(|class| class.name) {
            write!(f, "{}", class)?;
        }
        for enum_ in self.enum_.values().sorted_by_key(|class| class.name) {
            write!(f, "{}", enum_)?;
        }
        for function in self.function.values() {
            write!(f, "{}", function)?;
        }
        for error in self.error.iter() {
            writeln!(f, "class {}(Exception): ...", error)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StubInfo {
    pub modules: BTreeMap<String, Module>,
    pub pyproject: PyProject,
}

impl StubInfo {
    pub fn from_pyproject_toml(path: impl AsRef<Path>) -> Result<Self> {
        let pyproject = PyProject::parse_toml(path)?;
        Ok(Self::gather(pyproject))
    }

    fn default_module(&self) -> Result<&Module> {
        let default_module_name = self.pyproject.module_name();
        self.modules
            .get(default_module_name)
            .ok_or_else(|| anyhow!("Missing default module: {}", default_module_name))
    }

    fn gather(pyproject: PyProject) -> Self {
        let default_module_name = pyproject.module_name();
        let mut modules: BTreeMap<String, Module> = BTreeMap::new();

        for info in inventory::iter::<PyClassInfo> {
            let module_name = info
                .module
                .map(str::to_owned)
                .unwrap_or(default_module_name.to_string());
            let module = modules.entry(module_name).or_default();

            module
                .class
                .insert((info.struct_id)(), ClassDef::from(info));
        }

        for info in inventory::iter::<PyEnumInfo> {
            let module_name = info
                .module
                .map(str::to_owned)
                .unwrap_or(default_module_name.to_string());
            let module = modules.entry(module_name).or_default();
            module.enum_.insert((info.enum_id)(), EnumDef::from(info));
        }

        'methods_info: for info in inventory::iter::<PyMethodsInfo> {
            let struct_id = (info.struct_id)();
            for module in modules.values_mut() {
                if let Some(entry) = module.class.get_mut(&struct_id) {
                    for getter in info.getters {
                        entry.members.push(MemberDef {
                            name: getter.name,
                            r#type: (getter.r#type)(),
                        });
                    }
                    for method in info.methods {
                        entry.methods.push(MethodDef::from(method))
                    }
                    if let Some(new) = &info.new {
                        entry.new = Some(NewDef::from(new));
                    }
                    continue 'methods_info;
                }
            }
            unreachable!("Missing struct_id = {:?}", struct_id);
        }

        for info in inventory::iter::<PyFunctionInfo> {
            let module = modules
                .entry(
                    info.module
                        .map(str::to_string)
                        .unwrap_or(default_module_name.to_string()),
                )
                .or_default();
            module.function.insert(info.name, FunctionDef::from(info));
        }

        let default = modules.entry(default_module_name.to_string()).or_default();
        for info in inventory::iter::<PyErrorInfo> {
            default.error.insert(info.name);
        }

        Self { modules, pyproject }
    }

    pub fn generate(&self) -> Result<()> {
        if let Some(python_source) = self.pyproject.python_source() {
            log::trace!("`tool.maturin.python_source` exists in pyproject.toml. Regarded as Rust/Python mixed project.");

            for (name, module) in self.modules.iter() {
                let path: Vec<&str> = name.split('.').collect();
                let dest = if path.len() > 1 {
                    python_source.join(format!("{}.pyi", path.join("/")))
                } else {
                    python_source.join("__init__.pyi")
                };

                if let Some(dir) = dest.parent() {
                    fs::create_dir_all(dir)?;
                }
                let mut f = fs::File::create(&dest)?;
                write!(f, "{}", module)?;
                log::info!(
                    "Generate stub file of a module `{name}` at {dest}",
                    dest = dest.display()
                );
            }
        } else {
            log::trace!("`tool.maturin.python_source` is not in pyproject.toml. Regarded as pure Rust project.");

            let out_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
            if !out_dir.is_dir() {
                bail!("{} is not a directory", out_dir.display());
            }

            let name = self.pyproject.module_name();
            let dest = out_dir.join(format!("{}.pyi", name));

            let mut f = fs::File::create(&dest)?;
            let module = self.default_module()?;
            write!(f, "{}", module)?;

            log::info!(
                "Generate stub file of a module `{name}` at {dest}",
                dest = dest.display()
            );
        }
        Ok(())
    }
}
