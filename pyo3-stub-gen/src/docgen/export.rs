//! Export resolution for determining which items are publicly accessible

use crate::generate::Module;
use std::collections::{BTreeMap, BTreeSet};

/// Resolver for determining which items are exported from modules
pub struct ExportResolver<'a> {
    modules: &'a BTreeMap<String, Module>,
}

impl<'a> ExportResolver<'a> {
    pub fn new(modules: &'a BTreeMap<String, Module>) -> Self {
        Self { modules }
    }

    /// Resolve which items are exported from a module
    /// Rules:
    /// 1. If __all__ exists (re-exports or verbatim): use it
    /// 2. Otherwise: all non-underscore items
    /// 3. Add re-exported items
    /// 4. Add verbatim entries
    /// 5. Remove excluded items
    pub fn resolve_exports(&self, module: &Module) -> BTreeSet<String> {
        // TODO: Implement proper export resolution
        // For now, return all non-underscore items
        let mut exports = BTreeSet::new();

        // Add functions
        for name in module.function.keys() {
            if !name.starts_with('_') {
                exports.insert(name.to_string());
            }
        }

        // Add classes
        for class in module.class.values() {
            if !class.name.starts_with('_') {
                exports.insert(class.name.to_string());
            }
        }

        // Add enums
        for enum_ in module.enum_.values() {
            if !enum_.name.starts_with('_') {
                exports.insert(enum_.name.to_string());
            }
        }

        // Add type aliases
        for name in module.type_aliases.keys() {
            if !name.starts_with('_') {
                exports.insert(name.to_string());
            }
        }

        // Add variables
        for name in module.variables.keys() {
            if !name.starts_with('_') {
                exports.insert(name.to_string());
            }
        }

        exports
    }

    /// Build global map: item_fqn → module_where_exported
    /// For re-exports: map to re-exporting module
    pub fn build_export_map(&self) -> BTreeMap<String, String> {
        let mut export_map = BTreeMap::new();

        for (module_name, module) in self.modules {
            let exports = self.resolve_exports(module);

            for item_name in exports {
                let fqn = format!("{}.{}", module_name, item_name);
                export_map.insert(fqn, module_name.clone());
            }
        }

        export_map
    }
}
