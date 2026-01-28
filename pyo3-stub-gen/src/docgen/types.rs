//! Type expression rendering for documentation

use crate::docgen::ir::{DocTypeExpr, ItemKind, LinkTarget};
use crate::docgen::link::LinkResolver;
use crate::{ModuleRef, TypeInfo};
use std::collections::HashMap;

/// Type alias definition placeholder
pub struct TypeAliasDef;

/// Renderer for type expressions
pub struct TypeRenderer<'a> {
    link_resolver: &'a LinkResolver<'a>,
    current_module: &'a str,
    #[allow(dead_code)]
    type_aliases: &'a HashMap<String, TypeAliasDef>,
    default_module_name: &'a str,
}

impl<'a> TypeRenderer<'a> {
    pub fn new(
        link_resolver: &'a LinkResolver<'a>,
        current_module: &'a str,
        type_aliases: &'a HashMap<String, TypeAliasDef>,
        default_module_name: &'a str,
    ) -> Self {
        Self {
            link_resolver,
            current_module,
            type_aliases,
            default_module_name,
        }
    }

    /// Render a type expression
    /// 1. Check if this is a type alias - preserve name, don't expand
    /// 2. Strip module prefixes from display text
    /// 3. Resolve link target using LinkResolver
    /// 4. Recursively handle generic parameters
    pub fn render_type(&self, type_info: &TypeInfo) -> DocTypeExpr {
        let display = self.strip_module_prefix(&type_info.to_string());

        // Check if this is a simple type reference (not a compound expression)
        // If so, try to create a link target for it
        let link_target = self.try_create_link_target(type_info);

        DocTypeExpr {
            display,
            link_target,
            children: Vec::new(), // TODO: handle generics recursively
        }
    }

    /// Try to create a link target for a type
    /// Returns Some if this is a local type that should be linked
    fn try_create_link_target(&self, type_info: &TypeInfo) -> Option<LinkTarget> {
        // Only create links for simple type references (no generic params, no unions, etc.)
        // The name should be a simple identifier or qualified name
        if type_info.name.contains('[')
            || type_info.name.contains('|')
            || type_info.name.contains(',')
        {
            return None;
        }

        // Check if this type is from our package
        let source_module = type_info.source_module.as_ref()?;

        let fqn = match source_module {
            ModuleRef::Default => {
                // Type from default module
                format!("{}.{}", self.default_module_name, type_info.name)
            }
            ModuleRef::Named(module_path) => {
                // Type from named module - extract the bare type name
                // name might be like "main_mod.A" but we want just "A"
                let bare_name = type_info
                    .name
                    .split('.')
                    .next_back()
                    .unwrap_or(&type_info.name);
                format!("{}.{}", module_path, bare_name)
            }
        };

        // Determine item kind (assume Class for now, could be TypeAlias)
        // TODO: Track actual item kinds
        let kind = ItemKind::Class;

        Some(LinkTarget {
            fqn,
            doc_module: self.current_module.to_string(),
            kind,
        })
    }

    /// Strip module prefixes from type names
    /// Remove "typing.", "builtins.", "package.submod."
    /// Keep only bare names: "Optional[ClassA]" not "typing.Optional[sub_mod.ClassA]"
    fn strip_module_prefix(&self, type_name: &str) -> String {
        // Known external prefixes to strip
        let external_prefixes = &[
            "typing.",
            "builtins.",
            "collections.abc.",
            "typing_extensions.",
            "decimal.",
            "datetime.",
            "pathlib.",
        ];

        let mut result = String::new();
        let mut i = 0;
        let chars: Vec<char> = type_name.chars().collect();

        while i < chars.len() {
            // Check if we're at the start of a qualified name
            if i == 0
                || !chars[i - 1].is_alphanumeric() && chars[i - 1] != '_' && chars[i - 1] != '.'
            {
                let remaining: String = chars[i..].iter().collect();
                let mut matched = false;

                // Try to match external prefixes first
                for prefix in external_prefixes {
                    if remaining.starts_with(prefix) {
                        let after_prefix_idx = prefix.len();
                        if after_prefix_idx < remaining.len() {
                            let next_char = remaining.chars().nth(after_prefix_idx).unwrap();
                            if next_char.is_alphabetic() || next_char == '_' {
                                i += prefix.len();
                                matched = true;
                                break;
                            }
                        }
                    }
                }

                if matched {
                    continue;
                }

                // Try to match local package prefixes
                // Extract qualified identifier (e.g., "main_mod.A" or "pure.DataContainer")
                let ident_match = remaining
                    .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
                    .next();
                if let Some(ident) = ident_match {
                    if ident.contains('.') {
                        // This is a qualified name, check if it's from our package
                        let parts: Vec<&str> = ident.split('.').collect();
                        if parts.len() >= 2 {
                            // Check if the first part might be a module in our package
                            // by seeing if it starts with lowercase (modules are usually lowercase)
                            let first_part = parts[0];
                            let last_part = parts[parts.len() - 1];

                            // If it looks like a package.Type pattern, extract just the Type
                            if first_part
                                .chars()
                                .next()
                                .map(|c| c.is_lowercase())
                                .unwrap_or(false)
                                && last_part
                                    .chars()
                                    .next()
                                    .map(|c| c.is_uppercase())
                                    .unwrap_or(false)
                            {
                                // Skip to the last part
                                let prefix_len = ident.len() - last_part.len();
                                i += prefix_len;
                                continue;
                            }
                        }
                    }
                }
            }

            result.push(chars[i]);
            i += 1;
        }

        result
    }
}
