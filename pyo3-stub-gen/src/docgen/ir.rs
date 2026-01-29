//! Intermediate representation for documentation generation

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Root documentation package structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocPackage {
    pub name: String,
    pub modules: BTreeMap<String, DocModule>,
    /// Maps item FQN to the public module where it's exported
    pub export_map: BTreeMap<String, String>,
}

/// A single module's documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocModule {
    pub name: String,
    pub doc: String,
    pub items: Vec<DocItem>,
    pub submodules: Vec<String>,
}

/// A reference to a submodule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocSubmodule {
    pub name: String,
    pub doc: String,
    pub fqn: String,
}

/// A documented item (function, class, type alias, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum DocItem {
    Function(DocFunction),
    Class(DocClass),
    TypeAlias(DocTypeAlias),
    Variable(DocVariable),
    Module(DocSubmodule),
}

/// A function with all its overload signatures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocFunction {
    pub name: String,
    /// MyST-formatted docstring
    pub doc: String,
    /// ALL overload cases
    pub signatures: Vec<DocSignature>,
    pub is_async: bool,
    pub deprecated: Option<DeprecatedInfo>,
}

/// A single function signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocSignature {
    pub parameters: Vec<DocParameter>,
    pub return_type: Option<DocTypeExpr>,
}

/// A function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocParameter {
    pub name: String,
    pub type_: DocTypeExpr,
    pub default: Option<String>,
}

/// A type alias definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocTypeAlias {
    pub name: String,
    pub doc: String,
    pub definition: DocTypeExpr,
}

/// A module-level variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocVariable {
    pub name: String,
    pub doc: String,
    pub type_: Option<DocTypeExpr>,
}

/// A class definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocClass {
    pub name: String,
    pub doc: String,
    pub bases: Vec<DocTypeExpr>,
    pub methods: Vec<DocFunction>,
    pub attributes: Vec<DocAttribute>,
    pub deprecated: Option<DeprecatedInfo>,
}

/// A class attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocAttribute {
    pub name: String,
    pub doc: String,
    pub type_: Option<DocTypeExpr>,
}

/// Type expression with separate display and link target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocTypeExpr {
    /// Display text (stripped of module prefixes): "ClassA"
    pub display: String,
    /// Where to link (FQN after Haddock resolution)
    pub link_target: Option<LinkTarget>,
    /// Generic parameters (recursively)
    pub children: Vec<DocTypeExpr>,
}

/// Link target information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkTarget {
    /// Fully qualified name
    pub fqn: String,
    /// Module where the item is documented (after Haddock resolution)
    pub doc_module: String,
    /// Item kind
    pub kind: ItemKind,
}

/// Kind of documented item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemKind {
    Class,
    Function,
    TypeAlias,
    Variable,
    Module,
}

/// Deprecation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecatedInfo {
    pub since: Option<String>,
    pub note: Option<String>,
}
