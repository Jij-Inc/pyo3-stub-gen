use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::*};

pub fn parse_toml(path: impl AsRef<Path>) -> Result<PyProject> {
    let path = path.as_ref();
    if path.file_name() != Some("pyproject.toml".as_ref()) {
        bail!("{} is not a pyproject.toml", path.display())
    }
    let out = toml::de::from_str(&fs::read_to_string(path)?)?;
    Ok(out)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PyProject {
    pub project: Project,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
}
