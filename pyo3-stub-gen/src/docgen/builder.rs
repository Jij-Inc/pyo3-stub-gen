//! Builder for converting StubInfo to DocPackage

use crate::docgen::{
    export::ExportResolver,
    ir::{
        DeprecatedInfo, DocAttribute, DocClass, DocFunction, DocItem, DocModule, DocPackage,
        DocParameter, DocSignature, DocSubmodule, DocTypeAlias, DocTypeExpr, DocVariable,
    },
    types::TypeRenderer,
};
use crate::generate::StubInfo;
use crate::Result;
use std::collections::{BTreeMap, HashMap};

/// Check if module is hidden (any path component starts with '_')
fn is_hidden_module(module_name: &str) -> bool {
    module_name.split('.').any(|part| part.starts_with('_'))
}

/// Helper to check if item already exists in the list
fn matches_item_name(item: &DocItem, name: &str) -> bool {
    match item {
        DocItem::Function(f) => f.name == name,
        DocItem::Class(c) => c.name == name,
        DocItem::TypeAlias(t) => t.name == name,
        DocItem::Variable(v) => v.name == name,
        DocItem::Module(m) => m.name == name,
    }
}

/// Builder for converting StubInfo to DocPackage
pub struct DocPackageBuilder<'a> {
    stub_info: &'a StubInfo,
    export_resolver: ExportResolver<'a>,
    export_map: BTreeMap<String, String>,
    default_module_name: String,
}

impl<'a> DocPackageBuilder<'a> {
    pub fn new(stub_info: &'a StubInfo) -> Self {
        let export_resolver = ExportResolver::new(&stub_info.modules);
        let export_map = export_resolver.build_export_map();

        // Get the default module name from the first module (they all share the same default_module_name)
        let default_module_name = stub_info
            .modules
            .values()
            .next()
            .map(|m| m.default_module_name.clone())
            .unwrap_or_default();

        Self {
            stub_info,
            export_resolver,
            export_map,
            default_module_name,
        }
    }

    pub fn build(self) -> Result<DocPackage> {
        let mut modules = BTreeMap::new();

        for (module_name, module) in &self.stub_info.modules {
            // Skip modules with any component starting with '_'
            if is_hidden_module(module_name) {
                continue;
            }

            let doc_module = self.build_module(module_name, module)?;
            modules.insert(module_name.clone(), doc_module);
        }

        Ok(DocPackage {
            name: self.default_module_name.clone(),
            modules,
            export_map: self.export_map,
            config: self.stub_info.config.doc_gen.clone().unwrap_or_default(),
        })
    }

    fn build_module(&self, name: &str, module: &crate::generate::Module) -> Result<DocModule> {
        let exports = self.export_resolver.resolve_exports(module);
        let mut items = Vec::new();

        // Process functions - handle overloads (Requirement #1)
        for (func_name, func_defs) in &module.function {
            if exports.contains(*func_name) {
                items.push(self.build_function(name, func_defs)?);
            }
        }

        // Process type aliases (Requirement #2)
        for (alias_name, alias_def) in &module.type_aliases {
            if exports.contains(*alias_name) {
                items.push(self.build_type_alias(name, alias_def)?);
            }
        }

        // Process classes (sorted by name for deterministic output)
        let mut classes: Vec<_> = module.class.values().collect();
        classes.sort_by_key(|c| c.name);
        for class_def in classes {
            if exports.contains(class_def.name) {
                items.push(self.build_class(name, class_def)?);
            }
        }

        // Process enums (sorted by name for deterministic output)
        let mut enums: Vec<_> = module.enum_.values().collect();
        enums.sort_by_key(|e| e.name);
        for enum_def in enums {
            if exports.contains(enum_def.name) {
                items.push(self.build_enum_as_class(name, enum_def)?);
            }
        }

        // Process variables
        for (var_name, var_def) in &module.variables {
            if exports.contains(*var_name) {
                items.push(self.build_variable(name, var_def)?);
            }
        }

        // Process module re-exports (from reexport_module_members!)
        for re_export in &module.module_re_exports {
            if let Some(source_module) = self.stub_info.modules.get(&re_export.source_module) {
                for item_name in &re_export.items {
                    // Skip if already added (prefer directly-defined items)
                    if items.iter().any(|item| matches_item_name(item, item_name)) {
                        continue;
                    }

                    // Build re-exported item (link targets will be corrected later)
                    if let Some(item) = self.build_reexported_item(
                        &re_export.source_module,
                        source_module,
                        item_name,
                    )? {
                        items.push(item);
                    }
                }
            }
        }

        // Process submodules - convert to DocItem::Module entries
        for submod_name in &module.submodules {
            if !submod_name.starts_with('_') && exports.contains(submod_name) {
                // Construct FQN for the submodule
                let submod_fqn = if name.is_empty() {
                    submod_name.clone()
                } else {
                    format!("{}.{}", name, submod_name)
                };

                // Retrieve the submodule's doc from stub_info.modules
                let submod_doc = self
                    .stub_info
                    .modules
                    .get(&submod_fqn)
                    .map(|m| m.doc.clone())
                    .unwrap_or_default();

                items.push(DocItem::Module(DocSubmodule {
                    name: submod_name.clone(),
                    doc: submod_doc,
                    fqn: submod_fqn,
                }));
            }
        }

        // Correct all link targets and display text in all items
        // This ensures both directly-defined and re-exported items have correct references
        for item in &mut items {
            self.correct_link_targets(item, name);
        }

        Ok(DocModule {
            name: name.to_string(),
            doc: module.doc.clone(),
            items,
            submodules: module
                .submodules
                .iter()
                .filter(|s| !s.starts_with('_'))
                .cloned()
                .collect(),
        })
    }

    fn build_function(
        &self,
        module: &str,
        func_defs: &[crate::generate::FunctionDef],
    ) -> Result<DocItem> {
        // Requirement #1: Include ALL overload signatures
        let signatures: Vec<DocSignature> = func_defs
            .iter()
            .map(|def| self.build_signature(module, def))
            .collect::<Result<_>>()?;

        // Use first def's doc (they should all have same doc)
        let doc = func_defs
            .first()
            .map(|d| d.doc.to_string())
            .unwrap_or_default();

        let deprecated = func_defs.first().and_then(|d| {
            d.deprecated.as_ref().map(|dep| DeprecatedInfo {
                since: dep.since.map(|s| s.to_string()),
                note: dep.note.map(|s| s.to_string()),
            })
        });

        Ok(DocItem::Function(DocFunction {
            name: func_defs[0].name.to_string(),
            doc,
            signatures,
            is_async: func_defs[0].is_async,
            deprecated,
        }))
    }

    fn build_signature_from_params(
        &self,
        module: &str,
        parameters: &crate::generate::Parameters,
        return_type: &crate::TypeInfo,
    ) -> Result<DocSignature> {
        let type_aliases = HashMap::new();
        let link_resolver =
            crate::docgen::link::LinkResolver::new(&self.export_map, &self.default_module_name);
        let type_renderer = TypeRenderer::new(
            &link_resolver,
            module,
            &type_aliases,
            &self.default_module_name,
        );
        let default_parser =
            crate::docgen::default_parser::DefaultValueParser::new(&link_resolver, module);

        let params: Vec<DocParameter> = parameters
            .positional_only
            .iter()
            .chain(parameters.positional_or_keyword.iter())
            .chain(parameters.keyword_only.iter())
            .chain(parameters.varargs.iter())
            .chain(parameters.varkw.iter())
            .map(|param| DocParameter {
                name: param.name.to_string(),
                type_: type_renderer.render_type(&param.type_info),
                default: match &param.default {
                    crate::generate::ParameterDefault::None => None,
                    crate::generate::ParameterDefault::Expr(s) => {
                        // Parse default value and identify type references
                        Some(default_parser.parse(s, &param.type_info))
                    }
                },
            })
            .collect();

        let ret_type = Some(type_renderer.render_type(return_type));

        Ok(DocSignature {
            parameters: params,
            return_type: ret_type,
        })
    }

    fn build_signature(
        &self,
        module: &str,
        def: &crate::generate::FunctionDef,
    ) -> Result<DocSignature> {
        self.build_signature_from_params(module, &def.parameters, &def.r#return)
    }

    fn build_signature_from_method(
        &self,
        module: &str,
        def: &crate::generate::MethodDef,
    ) -> Result<DocSignature> {
        self.build_signature_from_params(module, &def.parameters, &def.r#return)
    }

    fn build_type_alias(
        &self,
        module: &str,
        alias: &crate::generate::TypeAliasDef,
    ) -> Result<DocItem> {
        // Requirement #2: Preserve alias definition + docstring
        let type_aliases = HashMap::new();
        let link_resolver =
            crate::docgen::link::LinkResolver::new(&self.export_map, &self.default_module_name);
        let type_renderer = TypeRenderer::new(
            &link_resolver,
            module,
            &type_aliases,
            &self.default_module_name,
        );

        Ok(DocItem::TypeAlias(DocTypeAlias {
            name: alias.name.to_string(),
            doc: alias.doc.to_string(),
            definition: type_renderer.render_type(&alias.type_),
        }))
    }

    fn build_class(&self, module: &str, class: &crate::generate::ClassDef) -> Result<DocItem> {
        let type_aliases = HashMap::new();
        let link_resolver =
            crate::docgen::link::LinkResolver::new(&self.export_map, &self.default_module_name);
        let type_renderer = TypeRenderer::new(
            &link_resolver,
            module,
            &type_aliases,
            &self.default_module_name,
        );

        let bases: Vec<DocTypeExpr> = class
            .bases
            .iter()
            .map(|base| type_renderer.render_type(base))
            .collect();

        let mut methods = Vec::new();
        for (method_name, method_overloads) in &class.methods {
            let signatures: Vec<DocSignature> = method_overloads
                .iter()
                .map(|method| self.build_signature_from_method(module, method))
                .collect::<Result<_>>()?;

            let deprecated = method_overloads.first().and_then(|d| {
                d.deprecated.as_ref().map(|dep| DeprecatedInfo {
                    since: dep.since.map(|s| s.to_string()),
                    note: dep.note.map(|s| s.to_string()),
                })
            });

            methods.push(DocFunction {
                name: method_name.to_string(),
                doc: method_overloads
                    .first()
                    .map(|m| m.doc.to_string())
                    .unwrap_or_default(),
                signatures,
                is_async: method_overloads
                    .first()
                    .map(|m| m.is_async)
                    .unwrap_or(false),
                deprecated,
            });
        }

        let attributes = Vec::new(); // TODO: implement attributes

        Ok(DocItem::Class(DocClass {
            name: class.name.to_string(),
            doc: class.doc.to_string(),
            bases,
            methods,
            attributes,
            deprecated: None, // ClassDef doesn't have deprecated field
        }))
    }

    fn build_enum_as_class(
        &self,
        _module: &str,
        enum_def: &crate::generate::EnumDef,
    ) -> Result<DocItem> {
        // Convert enum to class-like representation
        // EnumDef doesn't have bases field

        // Convert enum variants to DocAttributes
        let attributes: Vec<DocAttribute> = enum_def
            .variants
            .iter()
            .map(|(variant_name, variant_doc)| DocAttribute {
                name: (*variant_name).to_string(),
                doc: (*variant_doc).to_string(),
                type_: None, // Enum variants don't have explicit type annotations
            })
            .collect();

        Ok(DocItem::Class(DocClass {
            name: enum_def.name.to_string(),
            doc: enum_def.doc.to_string(),
            bases: Vec::new(), // Enums don't have bases in our structure
            methods: Vec::new(),
            attributes,
            deprecated: None,
        }))
    }

    fn build_variable(&self, module: &str, var: &crate::generate::VariableDef) -> Result<DocItem> {
        let type_aliases = HashMap::new();
        let link_resolver =
            crate::docgen::link::LinkResolver::new(&self.export_map, &self.default_module_name);
        let type_renderer = TypeRenderer::new(
            &link_resolver,
            module,
            &type_aliases,
            &self.default_module_name,
        );

        Ok(DocItem::Variable(DocVariable {
            name: var.name.to_string(),
            doc: String::new(), // VariableDef doesn't have doc field
            type_: Some(type_renderer.render_type(&var.type_)),
        }))
    }

    /// Build a re-exported item from a source module
    fn build_reexported_item(
        &self,
        source_module_name: &str,
        source_module: &crate::generate::Module,
        item_name: &str,
    ) -> Result<Option<DocItem>> {
        // Try functions
        if let Some(func_defs) = source_module.function.get(item_name) {
            return Ok(Some(self.build_function(source_module_name, func_defs)?));
        }

        // Try classes
        for class_def in source_module.class.values() {
            if class_def.name == item_name {
                return Ok(Some(self.build_class(source_module_name, class_def)?));
            }
        }

        // Try enums
        for enum_def in source_module.enum_.values() {
            if enum_def.name == item_name {
                return Ok(Some(
                    self.build_enum_as_class(source_module_name, enum_def)?,
                ));
            }
        }

        // Try type aliases
        if let Some(alias_def) = source_module.type_aliases.get(item_name) {
            return Ok(Some(self.build_type_alias(source_module_name, alias_def)?));
        }

        // Try variables
        if let Some(var_def) = source_module.variables.get(item_name) {
            return Ok(Some(self.build_variable(source_module_name, var_def)?));
        }

        Ok(None)
    }

    /// Correct link targets in a re-exported item to point to the target module
    fn correct_link_targets(&self, item: &mut DocItem, _target_module: &str) {
        match item {
            DocItem::Function(func) => {
                for sig in &mut func.signatures {
                    if let Some(ret) = &mut sig.return_type {
                        self.correct_type_expr(ret);
                    }
                    for param in &mut sig.parameters {
                        self.correct_type_expr(&mut param.type_);
                    }
                }
            }
            DocItem::Class(cls) => {
                for base in &mut cls.bases {
                    self.correct_type_expr(base);
                }
                for method in &mut cls.methods {
                    for sig in &mut method.signatures {
                        if let Some(ret) = &mut sig.return_type {
                            self.correct_type_expr(ret);
                        }
                        for param in &mut sig.parameters {
                            self.correct_type_expr(&mut param.type_);
                        }
                    }
                }
                for attr in &mut cls.attributes {
                    if let Some(type_) = &mut attr.type_ {
                        self.correct_type_expr(type_);
                    }
                }
            }
            DocItem::TypeAlias(alias) => {
                self.correct_type_expr(&mut alias.definition);
            }
            DocItem::Variable(var) => {
                if let Some(type_) = &mut var.type_ {
                    self.correct_type_expr(type_);
                }
            }
            DocItem::Module(_) => {}
        }
    }

    /// Correct a type expression to use export_map for link targets
    fn correct_type_expr(&self, type_expr: &mut DocTypeExpr) {
        if let Some(link_target) = &mut type_expr.link_target {
            // Try to find the correct export module and FQN
            // First try the original FQN
            let (exported_fqn, exported_module) =
                if let Some(module) = self.export_map.get(&link_target.fqn) {
                    (link_target.fqn.clone(), module.clone())
                } else {
                    // If not found, try to extract the type name and look for it under other modules
                    // e.g., "hidden_module_docgen_test._core.A" -> try "hidden_module_docgen_test.A"
                    if let Some(type_name) = link_target.fqn.split('.').next_back() {
                        // Try each module in export_map to find a match
                        if let Some((fqn, module)) = self
                            .export_map
                            .iter()
                            .find(|(fqn, _)| fqn.ends_with(&format!(".{}", type_name)))
                        {
                            (fqn.clone(), module.clone())
                        } else {
                            (link_target.fqn.clone(), link_target.doc_module.clone())
                        }
                    } else {
                        (link_target.fqn.clone(), link_target.doc_module.clone())
                    }
                };

            link_target.fqn = exported_fqn;
            link_target.doc_module = exported_module;

            // Update display text to strip internal module prefixes
            // e.g., "_core.A" -> "A"
            type_expr.display = self.strip_internal_module_prefix(&type_expr.display);
        }
        for child in &mut type_expr.children {
            self.correct_type_expr(child);
        }
    }

    /// Strip internal module prefixes (modules starting with '_') from display text
    /// e.g., "_core.A" -> "A", "_internal.Foo" -> "Foo"
    fn strip_internal_module_prefix(&self, display: &str) -> String {
        let mut result = String::new();
        let mut i = 0;
        let chars: Vec<char> = display.chars().collect();

        while i < chars.len() {
            // Check if we're at the start of a potential module prefix
            if i == 0 || !chars[i - 1].is_alphanumeric() && chars[i - 1] != '_' {
                // Try to match a pattern like "_module." or "_module.submodule."
                let remaining: String = chars[i..].iter().collect();

                // Look for pattern: _identifier followed by .
                if remaining.starts_with('_') {
                    // Find the next non-identifier character
                    let mut j = i + 1;
                    while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                        j += 1;
                    }

                    // If followed by a dot, this is a module prefix to strip
                    if j < chars.len() && chars[j] == '.' {
                        // Skip the module name and the dot
                        i = j + 1;
                        continue;
                    }
                }
            }

            result.push(chars[i]);
            i += 1;
        }

        result
    }
}
