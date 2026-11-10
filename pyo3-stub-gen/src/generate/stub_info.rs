use crate::{
    generate::{docstring::normalize_docstring, *},
    pyproject::{PyProject, StubGenConfig},
    type_info::*,
};
use anyhow::{Context, Result};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::*,
};

#[derive(Debug, Clone, PartialEq)]
pub struct StubInfo {
    pub modules: BTreeMap<String, Module>,
    pub python_root: PathBuf,
    /// Whether this is a mixed Python/Rust layout (has `python-source` in pyproject.toml)
    pub is_mixed_layout: bool,
    /// Configuration options for stub generation
    pub config: StubGenConfig,
    /// Directory containing pyproject.toml (for relative path calculations)
    pub pyproject_dir: Option<PathBuf>,
    /// The default module name (from `module-name` in pyproject.toml or `#[pymodule]`)
    pub default_module_name: String,
    /// The project name (from `project.name` in pyproject.toml)
    /// Used for documentation generation as the package display name
    pub project_name: String,
}

impl StubInfo {
    /// Initialize [StubInfo] from a `pyproject.toml` file in `CARGO_MANIFEST_DIR`.
    /// This is automatically set up by the [crate::define_stub_info_gatherer] macro.
    pub fn from_pyproject_toml(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let pyproject = PyProject::parse_toml(path)?;
        let mut config = pyproject.stub_gen_config();

        // Resolve doc_gen paths relative to pyproject.toml location
        if let Some(resolved_doc_gen) = pyproject.doc_gen_config_resolved() {
            config.doc_gen = Some(resolved_doc_gen);
        }

        let pyproject_dir = path.parent().map(|p| p.to_path_buf());

        let mut stub_info = StubInfoBuilder::from_pyproject_toml(pyproject, config).build()?;
        stub_info.pyproject_dir = pyproject_dir;
        Ok(stub_info)
    }

    /// Initialize [StubInfo] with a specific module name, project root, and configuration.
    /// This must be placed in your PyO3 library crate, i.e. the same crate where [inventory::submit]ted,
    /// not in the `gen_stub` executables due to [inventory]'s mechanism.
    pub fn from_project_root(
        default_module_name: String,
        project_root: PathBuf,
        is_mixed_layout: bool,
        config: StubGenConfig,
    ) -> Result<Self> {
        StubInfoBuilder::from_project_root(
            default_module_name,
            project_root,
            is_mixed_layout,
            config,
        )
        .build()
    }

    pub fn generate(&self) -> Result<()> {
        // Validate: Pure Rust layout can only have a single module
        if !self.is_mixed_layout && self.modules.len() > 1 {
            let module_names: Vec<_> = self.modules.keys().collect();
            anyhow::bail!(
                "Pure Rust layout does not support multiple modules or submodules. Found {} modules: {}. \
                 Please use mixed Python/Rust layout (add `python-source` to [tool.maturin] in pyproject.toml) \
                 if you need multiple modules or submodules.",
                self.modules.len(),
                module_names.iter().map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(", ")
            );
        }

        for (name, module) in self.modules.iter() {
            // Skip empty modules (nothing to generate)
            if module.is_empty() {
                continue;
            }

            // Convert dashes to underscores for Python compatibility
            let normalized_name = name.replace("-", "_");
            let path = normalized_name.replace(".", "/");

            if self.is_pyo3_generated(name) {
                // PyO3 module: generate .pyi stub file
                let dest = if self.is_mixed_layout {
                    self.python_root.join(&path).join("__init__.pyi")
                } else {
                    // Pure Rust: use single file at root
                    let package_name = normalized_name
                        .split('.')
                        .next()
                        .filter(|s| !s.is_empty())
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "Module name is empty after normalization: original name was `{name}`"
                            )
                        })?;
                    self.python_root.join(format!("{package_name}.pyi"))
                };

                self.write_stub_file(&dest, module)?;
            } else {
                // Pure Python module: needs __init__.py generation
                if !module.is_init_py_compatible() {
                    // Has PyO3 items but not under module-name path
                    anyhow::bail!(
                        "Module '{}' has PyO3 items (classes, functions, etc.) but is not under \
                         the PyO3 module path '{}'. Either move these items to a module under '{}', \
                         or check your module path configuration.",
                        name,
                        self.default_module_name,
                        self.default_module_name
                    );
                }

                if !self.config.generate_init_py.is_enabled_for(name) {
                    anyhow::bail!(
                        "Module '{}' is not a PyO3 module and requires `generate-init-py` to be enabled. \
                         Add `generate-init-py = true` or `generate-init-py = [\"{}\"]` to \
                         [tool.pyo3-stub-gen] in pyproject.toml.",
                        name,
                        name
                    );
                }

                // Generate __init__.py only (no .pyi - types resolve through re-exports)
                let dir = self.python_root.join(&path);
                if !dir.exists() {
                    fs::create_dir_all(&dir)?;
                }

                let init_py_dest = dir.join("__init__.py");
                let init_py_content = module.format_init_py();
                fs::write(&init_py_dest, init_py_content)?;
                log::info!(
                    "Generate __init__.py for module `{name}` at {dest}",
                    dest = init_py_dest.display()
                );
            }
        }

        // Generate documentation if configured
        if let Some(doc_config) = &self.config.doc_gen {
            self.generate_docs(doc_config)?;
        }

        Ok(())
    }

    fn write_stub_file(&self, dest: &std::path::Path, module: &module::Module) -> Result<()> {
        let dir = dest.parent().context("Cannot get parent directory")?;
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }

        let content = module.format_with_config(self.config.use_type_statement);
        fs::write(dest, content)?;
        log::info!(
            "Generate stub file of a module `{}` at {dest}",
            module.name,
            dest = dest.display()
        );
        Ok(())
    }

    /// Check if a module is part of the PyO3 shared library.
    ///
    /// In mixed layout, modules at or below `module-name` are considered part of the
    /// PyO3 shared library. Modules above `module-name` are Pure Python modules.
    fn is_pyo3_generated(&self, module: &str) -> bool {
        // In pure Rust layout, all modules are PyO3-generated
        if !self.is_mixed_layout {
            return true;
        }

        // In mixed layout, only modules at or below module-name are PyO3-generated
        let normalized_module = module.replace("-", "_");
        let normalized_module_name = self.default_module_name.replace("-", "_");

        normalized_module == normalized_module_name
            || normalized_module.starts_with(&format!("{}.", normalized_module_name))
    }

    fn generate_docs(&self, config: &crate::docgen::DocGenConfig) -> Result<()> {
        config.validate()?;

        log::info!("Generating API documentation...");

        // 1. Build DocPackage IR
        let doc_package = crate::docgen::builder::DocPackageBuilder::new(self).build()?;

        // 2. Render to JSON
        let json_output = crate::docgen::render::render_to_json(&doc_package)?;

        // 3. Write files
        fs::create_dir_all(&config.output_dir)?;
        fs::write(config.output_dir.join(&config.json_output), json_output)?;

        // 4. Copy Sphinx extension
        crate::docgen::render::copy_sphinx_extension(&config.output_dir)?;

        // 5. Generate RST files
        if config.separate_pages {
            crate::docgen::render::generate_module_pages(&doc_package, &config.output_dir, config)?;
            if config.generate_index {
                crate::docgen::render::generate_index_rst(
                    &doc_package,
                    &config.output_dir,
                    config,
                )?;
            }
            if config.separate_items {
                crate::docgen::render::generate_item_pages(&doc_package, &config.output_dir)?;
                log::info!("Generated separate .rst pages for each item");
            }
            log::info!("Generated separate .rst pages for each module");
        }

        log::info!("Generated API docs at {:?}", config.output_dir);
        Ok(())
    }
}

struct StubInfoBuilder {
    modules: BTreeMap<String, Module>,
    default_module_name: String,
    project_name: String,
    python_root: PathBuf,
    is_mixed_layout: bool,
    config: StubGenConfig,
}

impl StubInfoBuilder {
    fn from_pyproject_toml(pyproject: PyProject, config: StubGenConfig) -> Self {
        let is_mixed_layout = pyproject.python_source().is_some();
        let python_root = pyproject
            .python_source()
            .unwrap_or(PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()));

        Self {
            modules: BTreeMap::new(),
            default_module_name: pyproject.module_name().to_string(),
            project_name: pyproject.project.name.clone(),
            python_root,
            is_mixed_layout,
            config,
        }
    }

    fn from_project_root(
        default_module_name: String,
        project_root: PathBuf,
        is_mixed_layout: bool,
        config: StubGenConfig,
    ) -> Self {
        // Derive project name from default_module_name (take root component)
        let project_name = default_module_name
            .split('.')
            .next()
            .unwrap_or(&default_module_name)
            .to_string();

        Self {
            modules: BTreeMap::new(),
            default_module_name,
            project_name,
            python_root: project_root,
            is_mixed_layout,
            config,
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
        // Group children by parent, but only for PyO3-generated parent modules.
        //
        // In standard Python, `import main` does NOT automatically make `main.sub` accessible.
        // However, PyO3's `add_submodule` adds submodules as attributes of the parent module,
        // so `import main` makes `main.sub` accessible automatically.
        //
        // To reflect this PyO3 behavior in stub files, we generate `from . import sub` statements
        // for PyO3-generated parent modules. Pure Python parent modules don't need this.
        let mut parent_to_children: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        // For each existing module, collect parent-child relationships within PyO3-generated modules
        for module in self.modules.keys() {
            let path = module.split('.').collect::<Vec<_>>();

            // Generate all parent paths and their immediate children
            for i in 1..path.len() {
                let parent = path[..i].join(".");

                // Only collect if parent is PyO3-generated
                if self.is_pyo3_generated(&parent) {
                    let child = path[i].to_string();
                    parent_to_children.entry(parent).or_default().insert(child);
                }
            }
        }

        // Create or update all parent modules
        for (parent, children) in parent_to_children {
            let module = self.modules.entry(parent.clone()).or_default();
            module.name = parent;
            module.default_module_name = self.default_module_name.clone();
            module.submodules.extend(children);
        }
    }

    /// Check if a module is part of the PyO3 shared library.
    ///
    /// In mixed layout, modules at or below `module-name` are considered part of the
    /// PyO3 shared library. Modules above `module-name` are Pure Python modules.
    ///
    /// FIXME: Currently this uses `module-name` from pyproject.toml as a heuristic.
    /// Ideally, we should detect actual `add_submodule` calls in #[pymodule] functions
    /// to more accurately determine which modules are part of the PyO3 library.
    fn is_pyo3_generated(&self, module: &str) -> bool {
        // In pure Rust layout, all modules are PyO3-generated
        if !self.is_mixed_layout {
            return true;
        }

        // In mixed layout, only modules at or below module-name are PyO3-generated
        let normalized_module = module.replace("-", "_");
        let normalized_module_name = self.default_module_name.replace("-", "_");

        normalized_module == normalized_module_name
            || normalized_module.starts_with(&format!("{}.", normalized_module_name))
    }

    fn add_class(&mut self, info: &PyClassInfo) {
        let mut class_def = ClassDef::from(info);
        class_def.resolve_default_modules(&self.default_module_name);
        self.get_module(info.module)
            .class
            .insert((info.struct_id)(), class_def);
    }

    fn add_complex_enum(&mut self, info: &PyComplexEnumInfo) {
        let mut class_def = ClassDef::from(info);
        class_def.resolve_default_modules(&self.default_module_name);
        self.get_module(info.module)
            .class
            .insert((info.enum_id)(), class_def);
    }

    fn add_enum(&mut self, info: &PyEnumInfo) {
        self.get_module(info.module)
            .enum_
            .insert((info.enum_id)(), EnumDef::from(info));
    }

    fn add_function(&mut self, info: &PyFunctionInfo) -> Result<()> {
        // Clone default_module_name to avoid borrow checker issues
        let default_module_name = self.default_module_name.clone();

        let target = self
            .get_module(info.module)
            .function
            .entry(info.name)
            .or_default();

        // Validation: Check for multiple non-overload functions
        let mut new_func = FunctionDef::from(info);
        new_func.resolve_default_modules(&default_module_name);

        if !new_func.is_overload {
            let non_overload_count = target.iter().filter(|f| !f.is_overload).count();
            if non_overload_count > 0 {
                anyhow::bail!(
                    "Multiple functions with name '{}' found without @overload decorator. \
                     Please add @overload decorator to all variants.",
                    info.name
                );
            }
        }

        target.push(new_func);
        Ok(())
    }

    fn add_variable(&mut self, info: &PyVariableInfo) {
        self.get_module(Some(info.module))
            .variables
            .insert(info.name, VariableDef::from(info));
    }

    fn add_type_var(&mut self, info: &PyTypeVarInfo) {
        self.get_module(Some(info.module))
            .type_vars
            .insert(info.name, TypeVarDef::from(info));
    }

    fn add_type_alias(&mut self, info: &TypeAliasInfo) {
        self.get_module(Some(info.module))
            .type_aliases
            .insert(info.name, TypeAliasDef::from(info));
    }

    fn add_module_doc(&mut self, info: &ModuleDocInfo) {
        let raw_doc = (info.doc)();
        self.get_module(Some(info.module)).doc = normalize_docstring(&raw_doc);
    }

    fn add_module_export(&mut self, info: &ReexportModuleMembers) {
        use crate::type_info::ReexportItems;

        let (items, additional_items) = match info.items {
            ReexportItems::Wildcard => (Vec::new(), Vec::new()),
            ReexportItems::Explicit(items) => {
                (items.iter().map(|s| s.to_string()).collect(), Vec::new())
            }
            ReexportItems::WildcardPlus(additional) => (
                Vec::new(),
                additional.iter().map(|s| s.to_string()).collect(),
            ),
        };

        self.get_module(Some(info.target_module))
            .module_re_exports
            .push(ModuleReExport {
                source_module: info.source_module.to_string(),
                items,
                additional_items,
            });
    }

    fn add_verbatim_export(&mut self, info: &ExportVerbatim) {
        self.get_module(Some(info.target_module))
            .verbatim_all_entries
            .insert(info.name.to_string());
    }

    fn add_exclude(&mut self, info: &ExcludeFromAll) {
        self.get_module(Some(info.target_module))
            .excluded_all_entries
            .insert(info.name.to_string());
    }

    fn resolve_wildcard_re_exports(&mut self) -> Result<()> {
        // Collect wildcard re-exports and their resolved items for __all__
        // (module_name, re_export_idx, resolved_items, additional_items_to_merge)
        let mut resolutions: Vec<(String, usize, Vec<String>, Vec<String>)> = Vec::new();

        for (module_name, module) in &self.modules {
            for (idx, re_export) in module.module_re_exports.iter().enumerate() {
                if re_export.items.is_empty() {
                    // Wildcard - resolve items for __all__
                    if let Some(source_mod) = self.modules.get(&re_export.source_module) {
                        // Internal module - collect all public items that would be in __all__
                        let mut items = Vec::new();
                        for class in source_mod.class.values() {
                            if !class.name.starts_with('_') {
                                items.push(class.name.to_string());
                            }
                        }
                        for enum_ in source_mod.enum_.values() {
                            if !enum_.name.starts_with('_') {
                                items.push(enum_.name.to_string());
                            }
                        }
                        for func_name in source_mod.function.keys() {
                            if !func_name.starts_with('_') {
                                items.push(func_name.to_string());
                            }
                        }
                        for var_name in source_mod.variables.keys() {
                            if !var_name.starts_with('_') {
                                items.push(var_name.to_string());
                            }
                        }
                        for alias_name in source_mod.type_aliases.keys() {
                            if !alias_name.starts_with('_') {
                                items.push(alias_name.to_string());
                            }
                        }
                        for submod in &source_mod.submodules {
                            if !submod.starts_with('_') {
                                items.push(submod.to_string());
                            }
                        }
                        // Include additional_items (e.g., __version__) for WildcardPlus
                        let additional = re_export.additional_items.clone();
                        resolutions.push((module_name.clone(), idx, items, additional));
                    } else {
                        // External module - cannot resolve, error
                        anyhow::bail!(
                            "Cannot resolve wildcard re-export in module '{}': source module '{}' not found. \
                             Wildcard re-exports only work with internal modules.",
                            module_name,
                            re_export.source_module
                        );
                    }
                }
            }
        }

        // Apply resolutions (populate items for wildcard imports and merge additional items)
        for (module_name, idx, mut items, additional) in resolutions {
            // Merge additional items (for WildcardPlus)
            items.extend(additional);
            // Deduplicate items to avoid redundant imports like `from m import A, A`
            let mut seen = BTreeSet::new();
            items.retain(|item| seen.insert(item.clone()));
            if let Some(module) = self.modules.get_mut(&module_name) {
                module.module_re_exports[idx].items = items;
                module.module_re_exports[idx].additional_items.clear();
            }
        }

        Ok(())
    }

    fn add_methods(&mut self, info: &PyMethodsInfo) -> Result<()> {
        let struct_id = (info.struct_id)();
        for module in self.modules.values_mut() {
            if let Some(entry) = module.class.get_mut(&struct_id) {
                for attr in info.attrs {
                    entry.attrs.push(MemberDef {
                        name: attr.name,
                        r#type: (attr.r#type)(),
                        doc: attr.doc,
                        default: attr.default.map(|f| f()),
                        deprecated: attr.deprecated.clone(),
                    });
                }
                for getter in info.getters {
                    entry
                        .getter_setters
                        .entry(getter.name.to_string())
                        .or_default()
                        .0 = Some(MemberDef {
                        name: getter.name,
                        r#type: (getter.r#type)(),
                        doc: getter.doc,
                        default: getter.default.map(|f| f()),
                        deprecated: getter.deprecated.clone(),
                    });
                }
                for setter in info.setters {
                    entry
                        .getter_setters
                        .entry(setter.name.to_string())
                        .or_default()
                        .1 = Some(MemberDef {
                        name: setter.name,
                        r#type: (setter.r#type)(),
                        doc: setter.doc,
                        default: setter.default.map(|f| f()),
                        deprecated: setter.deprecated.clone(),
                    });
                }
                for method in info.methods {
                    let entries = entry.methods.entry(method.name.to_string()).or_default();

                    // Validation: Check for multiple non-overload methods
                    let new_method = MethodDef::from(method);
                    if !new_method.is_overload {
                        let non_overload_count = entries.iter().filter(|m| !m.is_overload).count();
                        if non_overload_count > 0 {
                            anyhow::bail!(
                                "Multiple methods with name '{}' in class '{}' found without @overload decorator. \
                                 Please add @overload decorator to all variants.",
                                method.name, entry.name
                            );
                        }
                    }

                    entries.push(new_method);
                }
                return Ok(());
            } else if let Some(entry) = module.enum_.get_mut(&struct_id) {
                for attr in info.attrs {
                    entry.attrs.push(MemberDef {
                        name: attr.name,
                        r#type: (attr.r#type)(),
                        doc: attr.doc,
                        default: attr.default.map(|f| f()),
                        deprecated: attr.deprecated.clone(),
                    });
                }
                for getter in info.getters {
                    entry.getters.push(MemberDef {
                        name: getter.name,
                        r#type: (getter.r#type)(),
                        doc: getter.doc,
                        default: getter.default.map(|f| f()),
                        deprecated: getter.deprecated.clone(),
                    });
                }
                for setter in info.setters {
                    entry.setters.push(MemberDef {
                        name: setter.name,
                        r#type: (setter.r#type)(),
                        doc: setter.doc,
                        default: setter.default.map(|f| f()),
                        deprecated: setter.deprecated.clone(),
                    });
                }
                for method in info.methods {
                    // Validation: Check for multiple non-overload methods
                    let new_method = MethodDef::from(method);
                    if !new_method.is_overload {
                        let non_overload_count = entry
                            .methods
                            .iter()
                            .filter(|m| m.name == method.name && !m.is_overload)
                            .count();
                        if non_overload_count > 0 {
                            anyhow::bail!(
                                "Multiple methods with name '{}' in enum '{}' found without @overload decorator. \
                                 Please add @overload decorator to all variants.",
                                method.name, entry.name
                            );
                        }
                    }

                    entry.methods.push(new_method);
                }
                return Ok(());
            }
        }
        unreachable!("Missing struct_id/enum_id = {:?}", struct_id);
    }

    fn build(mut self) -> Result<StubInfo> {
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
            self.add_function(info)?;
        }
        for info in inventory::iter::<PyVariableInfo> {
            self.add_variable(info);
        }
        for info in inventory::iter::<PyTypeVarInfo> {
            self.add_type_var(info);
        }
        for info in inventory::iter::<TypeAliasInfo> {
            self.add_type_alias(info);
        }
        for info in inventory::iter::<ModuleDocInfo> {
            self.add_module_doc(info);
        }
        // Sort PyMethodsInfo by source location for deterministic IndexMap insertion order
        let mut methods_infos: Vec<&PyMethodsInfo> = inventory::iter::<PyMethodsInfo>().collect();
        methods_infos.sort_by_key(|info| (info.file, info.line, info.column));
        for info in methods_infos {
            self.add_methods(info)?;
        }
        // Collect __all__ export directives
        for info in inventory::iter::<ReexportModuleMembers> {
            self.add_module_export(info);
        }
        for info in inventory::iter::<ExportVerbatim> {
            self.add_verbatim_export(info);
        }
        for info in inventory::iter::<ExcludeFromAll> {
            self.add_exclude(info);
        }
        self.register_submodules();

        // Resolve wildcard re-exports
        self.resolve_wildcard_re_exports()?;

        Ok(StubInfo {
            modules: self.modules,
            python_root: self.python_root,
            is_mixed_layout: self.is_mixed_layout,
            config: self.config,
            pyproject_dir: None, // Will be set by from_pyproject_toml()
            default_module_name: self.default_module_name,
            project_name: self.project_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_submodules_creates_empty_parent_modules() {
        let mut builder = StubInfoBuilder::from_project_root(
            "test_module".to_string(),
            "/tmp".into(),
            false,
            StubGenConfig::default(),
        );

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
        let mut builder = StubInfoBuilder::from_project_root(
            "root".to_string(),
            "/tmp".into(),
            false,
            StubGenConfig::default(),
        );

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

    #[test]
    fn test_pure_layout_rejects_multiple_modules() {
        // Pure Rust layout should reject multiple modules (whether submodules or top-level)
        let stub_info = StubInfo {
            modules: {
                let mut map = BTreeMap::new();
                map.insert(
                    "mymodule".to_string(),
                    Module {
                        name: "mymodule".to_string(),
                        default_module_name: "mymodule".to_string(),
                        // Add some content so module is not empty
                        doc: "Test module".to_string(),
                        ..Default::default()
                    },
                );
                map.insert(
                    "mymodule.sub".to_string(),
                    Module {
                        name: "mymodule.sub".to_string(),
                        default_module_name: "mymodule".to_string(),
                        // Add some content so module is not empty
                        doc: "Test submodule".to_string(),
                        ..Default::default()
                    },
                );
                map
            },
            python_root: PathBuf::from("/tmp"),
            is_mixed_layout: false,
            config: StubGenConfig::default(),
            pyproject_dir: None,
            default_module_name: "mymodule".to_string(),
            project_name: "mymodule".to_string(),
        };

        let result = stub_info.generate();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Pure Rust layout does not support multiple modules or submodules")
        );
    }
}
