//! JSON rendering and Sphinx extension embedding

use crate::docgen::ir::DocPackage;
use crate::Result;
use std::path::Path;

/// Render DocPackage to JSON string
pub fn render_to_json(package: &DocPackage) -> Result<String> {
    Ok(serde_json::to_string_pretty(package)?)
}

/// Copy the embedded Sphinx extension to the output directory
pub fn copy_sphinx_extension(output_dir: &Path) -> Result<()> {
    let extension_code = include_str!("sphinx_ext.py");
    let ext_path = output_dir.join("pyo3_stub_gen_ext.py");
    std::fs::write(ext_path, extension_code)?;
    Ok(())
}

/// Generate RST files for each module
pub fn generate_module_pages(package: &DocPackage, output_dir: &Path) -> Result<()> {
    // Sort modules to ensure consistent ordering
    let mut module_names: Vec<_> = package.modules.keys().collect();
    module_names.sort();

    for module_name in module_names {
        let rst_content = format!(
            "{}\n{}\n\n.. pyo3-api:: {}\n",
            module_name,
            "=".repeat(module_name.len()),
            module_name
        );

        // Convert module name to filename: mixed.main_mod -> mixed.main_mod.rst
        let filename = format!("{}.rst", module_name);
        let file_path = output_dir.join(&filename);

        std::fs::write(file_path, rst_content)?;
    }

    Ok(())
}

/// Generate index.rst that references all module pages
pub fn generate_index_rst(package: &DocPackage, output_dir: &Path) -> Result<()> {
    let mut content = String::new();

    // Title
    content.push_str(&format!(
        "{} API Reference\n{}\n\n",
        package.name,
        "=".repeat(package.name.len() + 14)
    ));

    content.push_str("This is the API reference documentation generated from Rust code using pyo3-stub-gen.\n\n");

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
