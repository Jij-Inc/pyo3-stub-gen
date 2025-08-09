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
        let mut all_parent_child_pairs: Vec<(String, String)> = Vec::new();

        // For each existing module, collect all parent-child relationships
        for module in self.modules.keys() {
            let path = module.split('.').collect::<Vec<_>>();

            // Generate all parent paths and their immediate children
            for i in 1..path.len() {
                let parent = path[..i].join(".");
                let child = path[i].to_string();
                all_parent_child_pairs.push((parent, child));
            }
        }

        // Group children by parent
        let mut parent_to_children: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (parent, child) in all_parent_child_pairs {
            parent_to_children.entry(parent).or_default().insert(child);
        }

        // Create or update all parent modules
        for (parent, children) in parent_to_children {
            let module = self.modules.entry(parent.clone()).or_default();
            module.name = parent;
            module.default_module_name = self.default_module_name.clone();
            module.submodules.extend(children);
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
        target.push(FunctionDef::from(info));
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
                        deprecated: attr.deprecated.clone(),
                    });
                }
                for getter in info.getters {
                    entry.getters.push(MemberDef {
                        name: getter.name,
                        r#type: (getter.r#type)(),
                        doc: getter.doc,
                        default: getter.default.map(|s| s.as_str()),
                        deprecated: getter.deprecated.clone(),
                    });
                }
                for setter in info.setters {
                    entry.setters.push(MemberDef {
                        name: setter.name,
                        r#type: (setter.r#type)(),
                        doc: setter.doc,
                        default: setter.default.map(|s| s.as_str()),
                        deprecated: setter.deprecated.clone(),
                    });
                }
                for method in info.methods {
                    let entries = entry.methods.entry(method.name.to_string()).or_default();
                    entries.push(MethodDef::from(method));
                }
                return;
            } else if let Some(entry) = module.enum_.get_mut(&struct_id) {
                for attr in info.attrs {
                    entry.attrs.push(MemberDef {
                        name: attr.name,
                        r#type: (attr.r#type)(),
                        doc: attr.doc,
                        default: attr.default.map(|s| s.as_str()),
                        deprecated: attr.deprecated.clone(),
                    });
                }
                for getter in info.getters {
                    entry.getters.push(MemberDef {
                        name: getter.name,
                        r#type: (getter.r#type)(),
                        doc: getter.doc,
                        default: getter.default.map(|s| s.as_str()),
                        deprecated: getter.deprecated.clone(),
                    });
                }
                for setter in info.setters {
                    entry.setters.push(MemberDef {
                        name: setter.name,
                        r#type: (setter.r#type)(),
                        doc: setter.doc,
                        default: setter.default.map(|s| s.as_str()),
                        deprecated: setter.deprecated.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_submodules_creates_empty_parent_modules() {
        let mut builder =
            StubInfoBuilder::from_project_root("test_module".to_string(), "/tmp".into());

        // Simulate a module with only submodules
        builder.modules.insert(
            "test_module.sub_mod".to_string(),
            Module {
                name: "test_module.sub_mod".to_string(),
                default_module_name: "test_module".to_string(),
                ..Default::default()
            },
        );

        builder.register_submodules();

        // Check that the empty parent module was created
        assert!(builder.modules.contains_key("test_module"));
        let parent_module = &builder.modules["test_module"];
        assert_eq!(parent_module.name, "test_module");
        assert!(parent_module.submodules.contains("sub_mod"));

        // Verify the submodule still exists
        assert!(builder.modules.contains_key("test_module.sub_mod"));
    }

    #[test]
    fn test_register_submodules_with_multiple_levels() {
        let mut builder = StubInfoBuilder::from_project_root("root".to_string(), "/tmp".into());

        // Simulate deeply nested modules
        builder.modules.insert(
            "root.level1.level2.deep_mod".to_string(),
            Module {
                name: "root.level1.level2.deep_mod".to_string(),
                default_module_name: "root".to_string(),
                ..Default::default()
            },
        );

        builder.register_submodules();

        // Check that all intermediate parent modules were created
        assert!(builder.modules.contains_key("root"));
        assert!(builder.modules.contains_key("root.level1"));
        assert!(builder.modules.contains_key("root.level1.level2"));
        assert!(builder.modules.contains_key("root.level1.level2.deep_mod"));

        // Check submodule relationships
        assert!(builder.modules["root"].submodules.contains("level1"));
        assert!(builder.modules["root.level1"].submodules.contains("level2"));
        assert!(builder.modules["root.level1.level2"]
            .submodules
            .contains("deep_mod"));
    }
}
