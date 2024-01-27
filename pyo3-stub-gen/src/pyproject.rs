use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::*};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PyProject {
    pub project: Project,
    pub tool: Option<Tool>,
}

impl PyProject {
    pub fn parse_toml(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if path.file_name() != Some("pyproject.toml".as_ref()) {
            bail!("{} is not a pyproject.toml", path.display())
        }
        let out = toml::de::from_str(&fs::read_to_string(path)?)?;
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
    pub fn python_source(&self) -> Option<&Path> {
        if let Some(tool) = &self.tool {
            if let Some(maturin) = &tool.maturin {
                if let Some(python_source) = &maturin.python_source {
                    return Some(Path::new(python_source));
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
