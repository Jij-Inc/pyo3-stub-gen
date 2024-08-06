//! This crate creates stub files in following two steps using [inventory] crate:
//!
//! Define type information in Rust code (or by proc-macro)
//! ---------------------------------------------------------
//! There are several types for storing information about Python classes and methods. For example,
//!
//! - [type_info::PyClassInfo] stores information for creating Python's class definition in stub file
//! - [type_info::PyMethodsInfo] stores information for creating Python's method definition in stub file
//! - and others in [type_info] module.
//!
//! ### Manual definition
//!
//! For better understanding of what happens in the background, let's define these information manually:
//!
//! ```
//! use pyo3::*;
//! use pyo3_stub_gen::type_info::*;
//!
//! // Usual PyO3 class definition
//! #[pyclass(module = "my_module", name = "MyClass")]
//! struct MyClass {
//!     #[pyo3(get)]
//!     name: String,
//!     #[pyo3(get)]
//!     description: Option<String>,
//! }
//!
//! // Submit type information for stub file generation to inventory manually
//! inventory::submit!{
//!     // Send information about Python class
//!     PyClassInfo {
//!         // Type ID of Rust struct (used to gathering phase discussed later)
//!         struct_id: std::any::TypeId::of::<MyClass>,
//!
//!         // Python module name. Since stub file is generated per modules,
//!         // this helps where the class definition should be placed.
//!         module: Some("my_module"),
//!
//!         // Python class name
//!         pyclass_name: "MyClass",
//!
//!         members: &[
//!             MemberInfo {
//!                 name: "name",
//!                 r#type: <String as IntoPy<PyObject>>::type_output,
//!             },
//!             MemberInfo {
//!                 name: "description",
//!                 r#type: <Option<String> as IntoPy<PyObject>>::type_output,
//!             },
//!         ],
//!         doc: "Docstring used in Python",
//!     }
//! }
//! ```
//!
//! Gathering type information and generating stub file
//! ----------------------------------------------------
//! To be written
//!

pub use inventory;
pub use pyo3_stub_gen_derive as derive; // re-export to use in generated code

pub mod generate;
pub mod pyproject;
pub mod type_info;

pub type Result<T> = anyhow::Result<T>;
pub use generate::StubInfo;
