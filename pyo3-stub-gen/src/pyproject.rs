//! `pyproject.toml` parser for reading `[tool.maturin]` configuration.
//!
//! ```
//! use pyo3_stub_gen::pyproject::PyProject;
//! use std::path::Path;
//!
//! let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
//! let pyproject = PyProject::parse_toml(
//!     root.join("examples/mixed/pyproject.toml")
//! ).unwrap();
//! ```

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::*};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PyProject {
    pub project: Project,
    pub tool: Option<Tool>,

    #[serde(skip)]
    toml_path: PathBuf,
}

impl PyProject {
    pub fn parse_toml(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if path.file_name() != Some("pyproject.toml".as_ref()) {
            bail!("{} is not a pyproject.toml", path.display())
        }
        let mut out: PyProject = toml::de::from_str(&fs::read_to_string(path)?)?;
        out.toml_path = path.to_path_buf();
        Ok(out)
    }

    pub fn module_name(&self) -> &str {
        if let Some(tool) = &self.tool {
            if let Some(maturin) = &tool.maturin {
                if let Some(module_name) = &maturin.module_name {
                    return module_name;
                }
            }
        }
        &self.project.name
    }

    /// Return `tool.maturin.python_source` if it exists, which means the project is a mixed Rust/Python project.
    pub fn python_source(&self) -> Option<PathBuf> {
        if let Some(tool) = &self.tool {
            if let Some(maturin) = &tool.maturin {
                if let Some(python_source) = &maturin.python_source {
                    if let Some(base) = self.toml_path.parent() {
                        return Some(base.join(python_source));
                    } else {
                        return Some(PathBuf::from(python_source));
                    }
                }
            }
        }
        None
    }

    /// Return whether to use Python 3.12+ `type` statement syntax for type aliases.
    /// Default is false (use pre-3.12 `TypeAlias` syntax).
    pub fn use_type_statement(&self) -> bool {
        self.tool
            .as_ref()
            .and_then(|t| t.pyo3_stub_gen.as_ref())
            .map(|config| config.use_type_statement)
            .unwrap_or(false)
    }

    /// Return doc-gen configuration if present in pyproject.toml
    pub fn doc_gen_config(&self) -> Option<crate::docgen::DocGenConfig> {
        self.tool
            .as_ref()
            .and_then(|t| t.pyo3_stub_gen.as_ref())
            .and_then(|config| config.doc_gen.clone())
    }

    /// Return doc-gen configuration with output_dir resolved relative to pyproject.toml directory
    pub fn doc_gen_config_resolved(&self) -> Option<crate::docgen::DocGenConfig> {
        if let Some(mut config) = self.doc_gen_config() {
            // Resolve output_dir relative to pyproject.toml directory
            // Only resolve if the path is relative (absolute paths stay unchanged)
            if config.output_dir.is_relative() {
                if let Some(base) = self.toml_path.parent() {
                    config.output_dir = base.join(&config.output_dir);
                }
            }
            Some(config)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tool {
    pub maturin: Option<Maturin>,
    #[serde(rename = "pyo3-stub-gen")]
    pub pyo3_stub_gen: Option<Pyo3StubGen>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Maturin {
    #[serde(rename = "python-source")]
    pub python_source: Option<String>,
    #[serde(rename = "module-name")]
    pub module_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pyo3StubGen {
    #[serde(rename = "use-type-statement", default)]
    pub use_type_statement: bool,
    #[serde(rename = "doc-gen")]
    pub doc_gen: Option<crate::docgen::DocGenConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_use_type_statement_true() {
        let toml_str = r#"
            [project]
            name = "test"

            [tool.pyo3-stub-gen]
            use-type-statement = true
        "#;
        let pyproject: PyProject = toml::from_str(toml_str).unwrap();
        assert_eq!(pyproject.use_type_statement(), true);
    }

    #[test]
    fn test_use_type_statement_false() {
        let toml_str = r#"
            [project]
            name = "test"

            [tool.pyo3-stub-gen]
            use-type-statement = false
        "#;
        let pyproject: PyProject = toml::from_str(toml_str).unwrap();
        assert_eq!(pyproject.use_type_statement(), false);
    }

    #[test]
    fn test_use_type_statement_default() {
        let toml_str = r#"
            [project]
            name = "test"
        "#;
        let pyproject: PyProject = toml::from_str(toml_str).unwrap();
        assert_eq!(pyproject.use_type_statement(), false);
    }

    #[test]
    fn test_use_type_statement_empty_config() {
        let toml_str = r#"
            [project]
            name = "test"

            [tool.pyo3-stub-gen]
        "#;
        let pyproject: PyProject = toml::from_str(toml_str).unwrap();
        assert_eq!(pyproject.use_type_statement(), false);
    }

    #[test]
    fn test_doc_gen_config_resolved_relative_path() {
        let toml_str = r#"
            [project]
            name = "test"

            [tool.pyo3-stub-gen.doc-gen]
            output-dir = "docs/api"
        "#;
        let mut pyproject: PyProject = toml::from_str(toml_str).unwrap();
        pyproject.toml_path = PathBuf::from("/project/root/pyproject.toml");

        let config = pyproject.doc_gen_config_resolved().unwrap();
        assert_eq!(config.output_dir, PathBuf::from("/project/root/docs/api"));
    }

    #[test]
    fn test_doc_gen_config_resolved_absolute_path() {
        let toml_str = r#"
            [project]
            name = "test"

            [tool.pyo3-stub-gen.doc-gen]
            output-dir = "/absolute/path/docs"
        "#;
        let mut pyproject: PyProject = toml::from_str(toml_str).unwrap();
        pyproject.toml_path = PathBuf::from("/project/root/pyproject.toml");

        let config = pyproject.doc_gen_config_resolved().unwrap();
        assert_eq!(config.output_dir, PathBuf::from("/absolute/path/docs"));
    }

    #[test]
    fn test_doc_gen_config_resolved_missing_config() {
        let toml_str = r#"
            [project]
            name = "test"
        "#;
        let mut pyproject: PyProject = toml::from_str(toml_str).unwrap();
        pyproject.toml_path = PathBuf::from("/project/root/pyproject.toml");

        assert!(pyproject.doc_gen_config_resolved().is_none());
    }
}
