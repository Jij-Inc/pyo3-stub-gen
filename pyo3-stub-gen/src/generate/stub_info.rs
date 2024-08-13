use crate::{generate::*, pyproject::PyProject, type_info::*};
use anyhow::{anyhow, bail, ensure, Result};
use std::{collections::BTreeMap, fs, io::Write, path::*};

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

        for info in inventory::iter::<PyErrorInfo> {
            let module = modules.entry(info.module.to_string()).or_default();
            module.error.insert(info.name, ErrorDef::from(info));
        }

        Self { modules, pyproject }
    }

    pub fn generate(&self) -> Result<()> {
        if let Some(python_source) = self.pyproject.python_source() {
            log::trace!("`tool.maturin.python_source` exists in pyproject.toml. Regarded as Rust/Python mixed project.");

            for (name, module) in self.modules.iter() {
                let path: Vec<&str> = name.split('.').collect();
                ensure!(!path.is_empty(), "Empty module name");
                let dest = if path.len() > 1 {
                    python_source.join(format!("{}.pyi", path.join("/")))
                } else {
                    python_source.join(path[0]).join("__init__.pyi")
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
