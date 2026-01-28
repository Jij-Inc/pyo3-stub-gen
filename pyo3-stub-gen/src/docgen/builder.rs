//! Builder for converting StubInfo to DocPackage

use crate::docgen::{
    export::ExportResolver,
    ir::{
        DeprecatedInfo, DocClass, DocFunction, DocItem, DocModule, DocPackage, DocParameter,
        DocSignature, DocTypeAlias, DocTypeExpr, DocVariable,
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
            name: self.default_module_name,
            modules,
            export_map: self.export_map,
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
                    crate::generate::ParameterDefault::Expr(s) => Some(s.clone()),
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

        Ok(DocItem::Class(DocClass {
            name: enum_def.name.to_string(),
            doc: enum_def.doc.to_string(),
            bases: Vec::new(), // Enums don't have bases in our structure
            methods: Vec::new(),
            attributes: Vec::new(),
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
}
