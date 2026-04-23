//! JSON rendering and Sphinx extension embedding

use crate::docgen::config::DocGenConfig;
use crate::docgen::ir::{DocItem, DocPackage};
use crate::Result;
use std::path::Path;

/// Render DocPackage to JSON string
pub fn render_to_json(package: &DocPackage) -> Result<String> {
    // Normalize for deterministic output
    let mut normalized = package.clone();
    normalized.normalize();
    Ok(serde_json::to_string_pretty(&normalized)?)
}

/// Copy the embedded Sphinx extension to the output directory
pub fn copy_sphinx_extension(output_dir: &Path) -> Result<()> {
    let extension_code = include_str!("sphinx_ext.py");
    let ext_path = output_dir.join("pyo3_stub_gen_ext.py");
    std::fs::write(ext_path, extension_code)?;
    Ok(())
}

/// Generate RST files for each module
pub fn generate_module_pages(
    package: &DocPackage,
    output_dir: &Path,
    config: &DocGenConfig,
) -> Result<()> {
    // Sort modules to ensure consistent ordering
    let mut module_names: Vec<_> = package.modules.keys().collect();
    module_names.sort();

    for module_name in module_names {
        let module = &package.modules[module_name];

        let mut rst_content = format!("{}\n{}\n\n", module_name, "=".repeat(module_name.len()),);

        if config.separate_items {
            // Summary directive instead of full rendering
            rst_content.push_str(&format!(".. pyo3-api-summary:: {}\n\n", module_name));

            // Collect all items that get their own pages
            let item_pages: Vec<String> = module
                .items
                .iter()
                .filter_map(|item| {
                    let name = match item {
                        DocItem::Class(c) => &c.name,
                        DocItem::Function(f) => &f.name,
                        DocItem::TypeAlias(t) => &t.name,
                        DocItem::Variable(v) => &v.name,
                        DocItem::Module(_) => return None,
                    };
                    Some(format!("   _items/{}.{}", module_name, name))
                })
                .collect();

            // Add hidden toctree referencing item pages
            if !item_pages.is_empty() {
                rst_content.push_str(".. toctree::\n");
                rst_content.push_str("   :hidden:\n\n");
                for page in &item_pages {
                    rst_content.push_str(page);
                    rst_content.push('\n');
                }
            }
        } else {
            rst_content.push_str(&format!(".. pyo3-api:: {}\n", module_name));
        }

        // Convert module name to filename: mixed.main_mod -> mixed.main_mod.rst
        let filename = format!("{}.rst", module_name);
        let file_path = output_dir.join(&filename);

        std::fs::write(file_path, rst_content)?;
    }

    Ok(())
}

/// Generate individual RST pages for each item (class, function, type alias, variable)
///
/// Item pages are placed in `_items/` subdirectory to avoid filename collisions
/// with module pages (e.g., a class `pkg.Foo` vs submodule `pkg.Foo`).
///
/// The `_items/` directory is cleared before generation to remove stale pages
/// from renamed or deleted items.
pub fn generate_item_pages(package: &DocPackage, output_dir: &Path) -> Result<()> {
    let items_dir = output_dir.join("_items");
    if items_dir.exists() {
        std::fs::remove_dir_all(&items_dir)?;
    }
    std::fs::create_dir_all(&items_dir)?;

    for (module_name, module) in &package.modules {
        for item in &module.items {
            let (item_name, directive) = match item {
                DocItem::Class(c) => (c.name.as_str(), "pyo3-api-class"),
                DocItem::Function(f) => (f.name.as_str(), "pyo3-api-function"),
                DocItem::TypeAlias(t) => (t.name.as_str(), "pyo3-api-type-alias"),
                DocItem::Variable(v) => (v.name.as_str(), "pyo3-api-variable"),
                DocItem::Module(_) => continue,
            };

            let rst_content = format!(
                "{}\n{}\n\n.. {}:: {} {}\n",
                item_name,
                "=".repeat(item_name.len()),
                directive,
                module_name,
                item_name,
            );

            let filename = format!("{}.{}.rst", module_name, item_name);
            std::fs::write(items_dir.join(&filename), rst_content)?;
        }
    }

    Ok(())
}

/// Generate index.rst that references all module pages
pub fn generate_index_rst(
    package: &DocPackage,
    output_dir: &Path,
    config: &DocGenConfig,
) -> Result<()> {
    let mut content = String::new();

    // Title - use configured title or default to "{package_name} API Reference"
    let title = if let Some(custom_title) = &config.index_title {
        if custom_title.is_empty() {
            "API Reference".to_string()
        } else {
            custom_title.clone()
        }
    } else {
        format!("{} API Reference", package.name)
    };

    content.push_str(&format!("{}\n{}\n\n", title, "=".repeat(title.len())));

    // Add intro message (configurable or default)
    if let Some(intro) = &config.intro_message {
        if !intro.is_empty() {
            content.push_str(intro);
            content.push_str("\n\n");
        }
        // Empty string -> skip intro entirely
    } else {
        // Default message when not configured
        content.push_str(
            "This is the API reference documentation generated from Rust code using `pyo3-stub-gen <https://github.com/Jij-Inc/pyo3-stub-gen>`_.\n\n",
        );
    }

    // Create toctree
    content.push_str(".. toctree::\n");
    content.push_str("   :maxdepth: 2\n");
    content.push_str("   :caption: Modules:\n\n");

    // Sort modules to ensure consistent ordering
    let mut module_names: Vec<_> = package.modules.keys().collect();
    module_names.sort();

    for module_name in module_names {
        content.push_str(&format!("   {}\n", module_name));
    }

    let index_path = output_dir.join("index.rst");
    std::fs::write(index_path, content)?;

    Ok(())
}
