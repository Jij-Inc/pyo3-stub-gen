//! Configuration for documentation generation

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for documentation generation from pyproject.toml
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocGenConfig {
    /// Output directory for generated documentation
    #[serde(rename = "output-dir", default = "default_output_dir")]
    pub output_dir: PathBuf,

    /// Name of the JSON output file
    #[serde(rename = "json-output", default = "default_json_output")]
    pub json_output: String,

    /// Generate separate .rst pages for each module (default: true)
    #[serde(rename = "separate-pages", default = "default_separate_pages")]
    pub separate_pages: bool,
}

impl Default for DocGenConfig {
    fn default() -> Self {
        Self {
            output_dir: default_output_dir(),
            json_output: default_json_output(),
            separate_pages: default_separate_pages(),
        }
    }
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("docs/api")
}

fn default_json_output() -> String {
    "api_reference.json".to_string()
}

fn default_separate_pages() -> bool {
    true
}
