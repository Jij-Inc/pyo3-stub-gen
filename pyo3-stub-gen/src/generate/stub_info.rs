use crate::{generate::*, pyproject::PyProject, type_info::*};
use anyhow::{Context, Result};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::*,
};

#[derive(Debug, Clone, PartialEq)]
pub struct StubInfo {
    pub modules: BTreeMap<String, Module>,
    pub python_root: PathBuf,
}

impl StubInfo {
    /// Initialize [StubInfo] from a `pyproject.toml` file in `CARGO_MANIFEST_DIR`.
    /// This is automatically set up by the [crate::define_stub_info_gatherer] macro.
    pub fn from_pyproject_toml(path: impl AsRef<Path>) -> Result<Self> {
        let pyproject = PyProject::parse_toml(path)?;
        Ok(StubInfoBuilder::from_pyproject_toml(pyproject).build())
    }

    /// Initialize [StubInfo] with a specific module name and project root.
    /// This must be placed in your PyO3 library crate, i.e. the same crate where [inventory::submit]ted,
    /// not in the `gen_stub` executables due to [inventory]'s mechanism.
    pub fn from_project_root(default_module_name: String, project_root: PathBuf) -> Result<Self> {
        Ok(StubInfoBuilder::from_project_root(default_module_name, project_root).build())
    }

    pub fn generate(&self) -> Result<()> {
        for (name, module) in self.modules.iter() {
            // Convert dashes to underscores for Python compatibility
            let normalized_name = name.replace("-", "_");
            let path = normalized_name.replace(".", "/");
            let dest = if module.submodules.is_empty() {
                self.python_root.join(format!("{path}.pyi"))
            } else {
                self.python_root.join(path).join("__init__.pyi")
            };

            let dir = dest.parent().context("Cannot get parent directory")?;
            if !dir.exists() {
                fs::create_dir_all(dir)?;
            }

            let mut f = fs::File::create(&dest)?;
            write!(f, "{module}")?;
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
    python_root: PathBuf,
    any_overloaded: bool,
}

impl StubInfoBuilder {
    fn from_pyproject_toml(pyproject: PyProject) -> Self {
        StubInfoBuilder::from_project_root(
            pyproject.module_name().to_string(),
            pyproject
                .python_source()
                .unwrap_or(PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())),
        )
    }

    fn from_project_root(default_module_name: String, project_root: PathBuf) -> Self {
        Self {
            modules: BTreeMap::new(),
            default_module_name,
            python_root: project_root,
            any_overloaded: false,
        }
    }

    fn get_module(&mut self, name: Option<&str>) -> &mut Module {
        let name = name.unwrap_or(&self.default_module_name).to_string();
        let module = self.modules.entry(name.clone()).or_default();
        module.name = name;
        module.default_module_name = self.default_module_name.clone();
        module
    }

    fn register_submodules(&mut self) {
        let mut map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for module in self.modules.keys() {
            let path = module.split('.').collect::<Vec<_>>();
            let n = path.len();
            if n <= 1 {
                continue;
            }
            map.entry(path[..n - 1].join("."))
                .or_default()
                .insert(path[n - 1].to_string());
        }
        for (parent, children) in map {
            if let Some(module) = self.modules.get_mut(&parent) {
                module.submodules.extend(children);
            }
        }
    }

    fn add_class(&mut self, info: &PyClassInfo) {
        self.get_module(info.module)
            .class
            .insert((info.struct_id)(), ClassDef::from(info));
    }

    fn add_complex_enum(&mut self, info: &PyComplexEnumInfo) {
        self.get_module(info.module)
            .class
            .insert((info.enum_id)(), ClassDef::from(info));
    }

    fn add_enum(&mut self, info: &PyEnumInfo) {
        self.get_module(info.module)
            .enum_
            .insert((info.enum_id)(), EnumDef::from(info));
    }

    fn add_function(&mut self, info: &PyFunctionInfo) {
        let target = self
            .get_module(info.module)
            .function
            .entry(info.name)
            .or_default();
        let is_overloaded = !target.is_empty();
        target.push(FunctionDef::from(info));
        self.any_overloaded |= is_overloaded;
    }

    fn add_error(&mut self, info: &PyErrorInfo) {
        self.get_module(Some(info.module))
            .error
            .insert(info.name, ErrorDef::from(info));
    }

    fn add_variable(&mut self, info: &PyVariableInfo) {
        self.get_module(Some(info.module))
            .variables
            .insert(info.name, VariableDef::from(info));
    }

    fn add_methods(&mut self, info: &PyMethodsInfo) {
        let struct_id = (info.struct_id)();
        for module in self.modules.values_mut() {
            if let Some(entry) = module.class.get_mut(&struct_id) {
                for attr in info.attrs {
                    entry.attrs.push(MemberDef {
                        name: attr.name,
                        r#type: (attr.r#type)(),
                        doc: attr.doc,
                        default: attr.default.map(|s| s.as_str()),
                    });
                }
                for getter in info.getters {
                    entry.getters.push(MemberDef {
                        name: getter.name,
                        r#type: (getter.r#type)(),
                        doc: getter.doc,
                        default: getter.default.map(|s| s.as_str()),
                    });
                }
                for setter in info.setters {
                    entry.setters.push(MemberDef {
                        name: setter.name,
                        r#type: (setter.r#type)(),
                        doc: setter.doc,
                        default: setter.default.map(|s| s.as_str()),
                    });
                }
                for method in info.methods {
                    entry
                        .methods
                        .entry(method.name.to_string())
                        .or_default()
                        .push(MethodDef::from(method))
                }
                return;
            } else if let Some(entry) = module.enum_.get_mut(&struct_id) {
                for attr in info.attrs {
                    entry.attrs.push(MemberDef {
                        name: attr.name,
                        r#type: (attr.r#type)(),
                        doc: attr.doc,
                        default: attr.default.map(|s| s.as_str()),
                    });
                }
                for getter in info.getters {
                    entry.getters.push(MemberDef {
                        name: getter.name,
                        r#type: (getter.r#type)(),
                        doc: getter.doc,
                        default: getter.default.map(|s| s.as_str()),
                    });
                }
                for setter in info.setters {
                    entry.setters.push(MemberDef {
                        name: setter.name,
                        r#type: (setter.r#type)(),
                        doc: setter.doc,
                        default: setter.default.map(|s| s.as_str()),
                    });
                }
                for method in info.methods {
                    entry.methods.push(MethodDef::from(method))
                }
                return;
            }
        }
        unreachable!("Missing struct_id/enum_id = {:?}", struct_id);
    }

    fn build(mut self) -> StubInfo {
        for info in inventory::iter::<PyClassInfo> {
            self.add_class(info);
        }
        for info in inventory::iter::<PyComplexEnumInfo> {
            self.add_complex_enum(info);
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
        for info in inventory::iter::<PyVariableInfo> {
            self.add_variable(info);
        }
        for info in inventory::iter::<PyMethodsInfo> {
            self.add_methods(info);
        }
        self.register_submodules();
        StubInfo {
            modules: self.modules,
            python_root: self.python_root,
        }
    }
}
