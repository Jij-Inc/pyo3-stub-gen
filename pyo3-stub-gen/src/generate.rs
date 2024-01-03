//! Generate Python typing stub file a.k.a. `*.pyi` file.

use crate::type_info::*;

use anyhow::Result;
use itertools::Itertools;
use pyo3::inspect::types::TypeInfo;
use std::{
    any::TypeId,
    collections::{BTreeMap, BTreeSet},
    env, fmt, fs,
    io::Write,
    path::*,
};

fn indent() -> &'static str {
    "    "
}

#[derive(Debug, Clone, PartialEq)]
struct Arg {
    name: &'static str,
    r#type: TypeInfo,
}

impl Arg {
    fn from_info(info: &ArgInfo) -> Self {
        Self {
            name: info.name,
            r#type: (info.r#type)(),
        }
    }
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ReturnTypeInfo {
    r#type: TypeInfo,
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

#[derive(Debug, Clone, PartialEq)]
struct MethodDef {
    name: &'static str,
    args: Vec<Arg>,
    signature: Option<&'static str>,
    r#return: ReturnTypeInfo,
    doc: &'static str,
    is_static: bool,
    is_class: bool,
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

#[derive(Debug, Clone, PartialEq)]
struct MemberDef {
    name: &'static str,
    r#type: TypeInfo,
}

impl fmt::Display for MemberDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent = indent();
        writeln!(f, "{indent}{}: {}", self.name, self.r#type)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct NewDef {
    args: Vec<Arg>,
    signature: Option<&'static str>,
}

impl NewDef {
    fn from_info(info: &NewInfo) -> Self {
        Self {
            args: info.args.iter().map(Arg::from_info).collect(),
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

#[derive(Debug, Clone, PartialEq)]
struct ClassDef {
    name: &'static str,
    doc: &'static str,
    new: Option<NewDef>,
    members: Vec<MemberDef>,
    methods: Vec<MethodDef>,
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

#[derive(Debug, Clone, PartialEq)]
struct EnumDef {
    name: &'static str,
    doc: &'static str,
    variants: &'static [&'static str],
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

#[derive(Debug, Clone, PartialEq)]
struct FunctionDef {
    name: &'static str,
    args: Vec<Arg>,
    r#return: ReturnTypeInfo,
    signature: Option<&'static str>,
    doc: &'static str,
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

#[derive(Debug, Clone, PartialEq, Default)]
struct Module {
    class: BTreeMap<TypeId, ClassDef>,
    enum_: BTreeMap<TypeId, EnumDef>,
    function: BTreeMap<&'static str, FunctionDef>,
    error: BTreeSet<&'static str>,
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

/// Gather metadata generated by proc-macros
fn gather() -> Result<BTreeMap<String, Module>> {
    let mut modules: BTreeMap<String, Module> = BTreeMap::new();

    for info in inventory::iter::<PyClassInfo> {
        let module_name = info.module.map(str::to_owned).unwrap_or(pkg_name());
        let module = modules.entry(module_name).or_default();

        module.class.insert(
            (info.struct_id)(),
            ClassDef {
                name: info.pyclass_name,
                new: None,
                doc: info.doc,
                methods: Vec::new(),
                members: info
                    .members
                    .iter()
                    .map(|info| MemberDef {
                        name: info.name,
                        r#type: (info.r#type)(),
                    })
                    .collect(),
            },
        );
    }

    for info in inventory::iter::<PyEnumInfo> {
        let module_name = info.module.map(str::to_owned).unwrap_or(pkg_name());
        let module = modules.entry(module_name).or_default();
        module.enum_.insert(
            (info.enum_id)(),
            EnumDef {
                name: info.pyclass_name,
                doc: info.doc,
                variants: info.variants,
            },
        );
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
                    entry.methods.push(MethodDef {
                        name: method.name,
                        args: method.args.iter().map(Arg::from_info).collect(),
                        signature: method.signature,
                        r#return: (method.r#return)().into(),
                        doc: method.doc,
                        is_class: method.is_class,
                        is_static: method.is_static,
                    })
                }
                if let Some(new) = &info.new {
                    entry.new = Some(NewDef::from_info(new));
                }
                continue 'methods_info;
            }
        }
        unreachable!("Missing struct_id = {:?}", struct_id);
    }

    for info in inventory::iter::<PyFunctionInfo> {
        let module = modules
            .entry(info.module.map(str::to_string).unwrap_or(pkg_name()))
            .or_default();
        module.function.insert(
            info.name,
            FunctionDef {
                name: info.name,
                args: info.args.iter().map(Arg::from_info).collect(),
                r#return: (info.r#return)().into(),
                doc: info.doc,
                signature: info.signature,
            },
        );
    }

    let default = modules.entry(pkg_name()).or_default();
    for info in inventory::iter::<PyErrorInfo> {
        default.error.insert(info.name);
    }

    Ok(modules)
}

fn pkg_name() -> String {
    env::var("CARGO_PKG_NAME").unwrap()
}

pub fn generate(python_root: &Path) -> Result<()> {
    for (name, module) in gather()?.iter() {
        let path: Vec<&str> = name.split('.').collect();
        let dest = if path.len() > 1 {
            python_root.join(format!("{}.pyi", path[1..].join("/")))
        } else {
            python_root.join("__init__.pyi")
        };

        if let Some(dir) = dest.parent() {
            fs::create_dir_all(dir)?;
        }
        let mut f = fs::File::create(dest)?;
        writeln!(f, "# This file is automatically generated by gen_stub.rs\n")?;
        writeln!(f, "from typing import final, Any, List, Dict")?;
        writeln!(f, "from enum import Enum, auto")?;
        writeln!(f)?;
        write!(f, "{}", module)?;
    }
    Ok(())
}
