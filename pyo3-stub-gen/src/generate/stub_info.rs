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
        Ok(StubInfoBuilder::new(pyproject).build())
    }

    fn default_module(&self) -> Result<&Module> {
        let default_module_name = self.pyproject.module_name();
        self.modules
            .get(default_module_name)
            .ok_or_else(|| anyhow!("Missing default module: {}", default_module_name))
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

struct StubInfoBuilder {
    modules: BTreeMap<String, Module>,
    default_module_name: String,
    pyproject: PyProject,
}

impl StubInfoBuilder {
    fn new(pyproject: PyProject) -> Self {
        Self {
            modules: BTreeMap::new(),
            default_module_name: pyproject.module_name().to_string(),
            pyproject,
        }
    }

    fn get_module(&mut self, name: Option<&str>) -> &mut Module {
        if let Some(name) = name {
            let module = self.modules.entry(name.to_string()).or_default();
            module.default_module_name = Some(self.default_module_name.clone());
            return module;
        } else {
            self.modules
                .entry(self.default_module_name.clone())
                .or_default()
        }
    }

    fn add_class(&mut self, info: &PyClassInfo) {
        self.get_module(info.module)
            .class
            .insert((info.struct_id)(), ClassDef::from(info));
    }

    fn add_enum(&mut self, info: &PyEnumInfo) {
        self.get_module(info.module)
            .enum_
            .insert((info.enum_id)(), EnumDef::from(info));
    }

    fn add_function(&mut self, info: &PyFunctionInfo) {
        self.get_module(info.module)
            .function
            .insert(info.name, FunctionDef::from(info));
    }

    fn add_error(&mut self, info: &PyErrorInfo) {
        self.get_module(Some(info.module))
            .error
            .insert(info.name, ErrorDef::from(info));
    }

    fn add_methods(&mut self, info: &PyMethodsInfo) {
        let struct_id = (info.struct_id)();
        for module in self.modules.values_mut() {
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
                return;
            }
        }
        unreachable!("Missing struct_id = {:?}", struct_id);
    }

    fn build(mut self) -> StubInfo {
        for info in inventory::iter::<PyClassInfo> {
            self.add_class(info);
        }
        for info in inventory::iter::<PyEnumInfo> {
            self.add_enum(info);
        }
        for info in inventory::iter::<PyFunctionInfo> {
            self.add_function(info);
        }
        for info in inventory::iter::<PyErrorInfo> {
            self.add_error(info);
        }
        for info in inventory::iter::<PyMethodsInfo> {
            self.add_methods(info);
        }
        StubInfo {
            modules: self.modules,
            pyproject: self.pyproject,
        }
    }
}
