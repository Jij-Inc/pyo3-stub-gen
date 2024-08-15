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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tool {
    pub maturin: Option<Maturin>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Maturin {
    #[serde(rename = "python-source")]
    pub python_source: Option<String>,
    #[serde(rename = "module-name")]
    pub module_name: Option<String>,
}
