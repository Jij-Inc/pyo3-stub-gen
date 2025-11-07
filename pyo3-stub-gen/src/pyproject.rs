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

    /// Return the output directory for stub files.
    /// Uses `tool.pyo3-stub-gen.output-dir` if specified
    pub fn output_dir(&self) -> Option<PathBuf> {
        if let Some(tool) = &self.tool {
            if let Some(config) = &tool.pyo3_stub_gen {
                if let Some(output_dir) = &config.output_dir {
                    return Some(
                        self.toml_path
                            .parent()
                            .map(|base| base.join(output_dir))
                            .unwrap_or_else(|| PathBuf::from(output_dir)),
                    );
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
    #[serde(rename = "output-dir")]
    pub output_dir: Option<String>,
}

#[test]
fn test_pyo3_stub_gen_config() {
    let toml_content = r#"
[project]
name = "test-package"

[tool.pyo3-stub-gen]
output-dir = "stubs"
"#;

    let pyproject: PyProject = toml::from_str(toml_content).unwrap();

    assert_eq!(pyproject.project.name, "test-package");

    let tool = pyproject.tool.as_ref().unwrap();
    let pyo3_stub_gen = tool.pyo3_stub_gen.as_ref().unwrap();

    assert_eq!(pyo3_stub_gen.output_dir.as_deref(), Some("stubs"));
}

#[test]
fn test_pyo3_stub_gen_config_optional() {
    let toml_content = r#"
[project]
name = "test-package"
"#;

    let pyproject: PyProject = toml::from_str(toml_content).unwrap();

    assert_eq!(pyproject.project.name, "test-package");
    assert!(pyproject.tool.is_none() || pyproject.tool.as_ref().unwrap().pyo3_stub_gen.is_none());
}

#[test]
fn test_pyo3_stub_gen_with_maturin() {
    let toml_content = r#"
[project]
name = "test-package"

[tool.maturin]
python-source = "python"
module-name = "custom_module"

[tool.pyo3-stub-gen]
output-dir = "stubs"
"#;

    let pyproject: PyProject = toml::from_str(toml_content).unwrap();

    assert_eq!(pyproject.project.name, "test-package");

    let tool = pyproject.tool.as_ref().unwrap();

    // Check maturin config
    let maturin = tool.maturin.as_ref().unwrap();
    assert_eq!(maturin.python_source.as_deref(), Some("python"));
    assert_eq!(maturin.module_name.as_deref(), Some("custom_module"));

    // Check pyo3-stub-gen config
    let pyo3_stub_gen = tool.pyo3_stub_gen.as_ref().unwrap();
    assert_eq!(pyo3_stub_gen.output_dir.as_deref(), Some("stubs"));
}

#[test]
fn test_pyo3_stub_gen_output_dir_only() {
    // Test with only output-dir specified
    let toml_content = r#"
[project]
name = "test-package"

[tool.pyo3-stub-gen]
output-dir = "stubs"
"#;

    let pyproject: PyProject = toml::from_str(toml_content).unwrap();

    let tool = pyproject.tool.as_ref().unwrap();
    let pyo3_stub_gen = tool.pyo3_stub_gen.as_ref().unwrap();

    assert_eq!(pyo3_stub_gen.output_dir.as_deref(), Some("stubs"));
}

#[test]
fn test_output_dir_method_prioritizes_pyo3_stub_gen() {
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    // Create a temporary directory with a pyproject.toml
    let temp_dir = TempDir::new().unwrap();
    let toml_path = temp_dir.path().join("pyproject.toml");

    let toml_content = r#"
[project]
name = "test-package"

[tool.maturin]
python-source = "python"

[tool.pyo3-stub-gen]
output-dir = "stubs"
"#;

    let mut file = fs::File::create(&toml_path).unwrap();
    file.write_all(toml_content.as_bytes()).unwrap();
    drop(file);

    let pyproject = PyProject::parse_toml(&toml_path).unwrap();

    // output_dir() should return the pyo3-stub-gen output-dir
    let output = pyproject.output_dir().unwrap();
    assert_eq!(output, temp_dir.path().join("stubs"));
}

#[test]
fn test_output_dir_method_returns_none() {
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    // Create a temporary directory with a pyproject.toml
    let temp_dir = TempDir::new().unwrap();
    let toml_path = temp_dir.path().join("pyproject.toml");

    let toml_content = r#"
[project]
name = "test-package"

[tool.maturin]
python-source = "python"
"#;

    let mut file = fs::File::create(&toml_path).unwrap();
    file.write_all(toml_content.as_bytes()).unwrap();
    drop(file);

    let pyproject = PyProject::parse_toml(&toml_path).unwrap();

    // output_dir() should be None

    let output = pyproject.output_dir();
    assert_eq!(output, None);
}

#[test]
fn test_output_dir_method_returns_none_when_no_config() {
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    // Create a temporary directory with a minimal pyproject.toml
    let temp_dir = TempDir::new().unwrap();
    let toml_path = temp_dir.path().join("pyproject.toml");

    let toml_content = r#"
[project]
name = "test-package"
"#;

    let mut file = fs::File::create(&toml_path).unwrap();
    file.write_all(toml_content.as_bytes()).unwrap();
    drop(file);

    let pyproject = PyProject::parse_toml(&toml_path).unwrap();

    // output_dir() should return None
    assert!(pyproject.output_dir().is_none());
}
