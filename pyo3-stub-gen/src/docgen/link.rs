//! Haddock-style link resolution for documentation

use crate::docgen::ir::ItemKind;
use std::collections::HashMap;

/// Link resolver implementing Haddock-style resolution
pub struct LinkResolver<'a> {
    export_map: &'a HashMap<String, String>,
    package_name: &'a str,
}

impl<'a> LinkResolver<'a> {
    pub fn new(export_map: &'a HashMap<String, String>, package_name: &'a str) -> Self {
        Self {
            export_map,
            package_name,
        }
    }

    /// Resolve a link using Haddock rules:
    /// 1. If exported from current_module (incl. re-exports): link to current
    /// 2. Else if exported from public module: link there
    /// 3. Else (private module only): no link
    ///
    /// Returns (doc_module, item_kind) if linkable, None otherwise
    pub fn resolve_link(&self, item_fqn: &str, current_module: &str) -> Option<(String, ItemKind)> {
        // Check if item is exported from current module
        if let Some(export_module) = self.export_map.get(item_fqn) {
            if export_module == current_module {
                // Item is exported from current module - link to it
                return Some((current_module.to_string(), ItemKind::Class)); // TODO: determine actual kind
            }

            // Check if the export module is public
            if !self.is_private_module(export_module) {
                return Some((export_module.clone(), ItemKind::Class)); // TODO: determine actual kind
            }
        }

        // Item is only in private modules - no link
        None
    }

    /// Check if a module is private (has underscore segments)
    fn is_private_module(&self, module_name: &str) -> bool {
        module_name.split('.').any(|part| part.starts_with('_'))
    }
}
