//! This crate creates stub files in following two steps using [inventory] crate:
//!
//! Define type information in Rust code (or by proc-macro)
//! ---------------------------------------------------------
//! The first step is to define Python type information in Rust code. [type_info] module provides several structs, for example:
//!
//! - [type_info::PyFunctionInfo] stores information of Python function, i.e. the name of the function, arguments and its types, return type, etc.
//! - [type_info::PyClassInfo] stores information for Python class definition, i.e. the name of the class, members and its types, methods, etc.
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
//! // Submit type information for stub file generation to inventory
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
//! Roughly speaking, the above corresponds a following stub file `my_module.pyi`:
//!
//! ```python
//! class MyClass:
//!     """
//!     Docstring used in Python
//!     """
//!     name: str
//!     description: Optional[str]
//! ```
//!
//! We want to generate this [type_info::PyClassInfo] section automatically from `MyClass` Rust struct definition.
//! This is done by using `#[gen_stub_pyclass]` proc-macro:
//!
//! ```
//! use pyo3::*;
//! use pyo3_stub_gen::{type_info::*, derive::gen_stub_pyclass};
//!
//! // Usual PyO3 class definition
//! #[gen_stub_pyclass]
//! #[pyclass(module = "my_module", name = "MyClass")]
//! struct MyClass {
//!     #[pyo3(get)]
//!     name: String,
//!     #[pyo3(get)]
//!     description: Option<String>,
//! }
//! ```
//!
//! Since proc-macro is a converter from Rust code to Rust code, the output must be a Rust code.
//! However, we need to gather these [type_info::PyClassInfo] definitions to generate stub files,
//! and the above [inventory::submit] is for it.
//!
//! Gathering type information and generating stub file
//! ----------------------------------------------------
//! [inventory::iter] makes it possible to gather distributed [type_info::PyClassInfo] in the crate into a single place.
//!
//! [generate] module provides structs implementing [std::fmt::Display] to generate corresponding parts of stub file.
//! For example, [generate::MethodDef] generates Python class method definition as follows:
//!
//! ```rust
//! use pyo3::inspect::types::TypeInfo;
//! use pyo3_stub_gen::generate::*;
//!
//! let method = MethodDef {
//!     name: "foo",
//!     args: vec![Arg { name: "x", r#type: TypeInfo::builtin("int") }],
//!     signature: None,
//!     r#return: ReturnTypeInfo { r#type: TypeInfo::builtin("int") },
//!     doc: "This is a foo method.",
//!     is_static: false,
//!     is_class: false,
//! };
//!
//! assert_eq!(
//!     method.to_string().trim(),
//!     r#"
//!     def foo(self, x:int) -> int:
//!         r"""
//!         This is a foo method.
//!         """
//!         ...
//!     "#.trim()
//! );
//! ```
//!
//! [generate::ClassDef] generates Python class definition using [generate::MethodDef] and others, and other `*Def` structs works as well.
//!
//! [generate::Module] consists of `*Def` structs and yields an entire stub file `*.pyi` for a single Python (sub-)module, i.e. a shared library build by PyO3.
//! [generate::Module]s are created as a part of [StubInfo], which merges [type_info::PyClassInfo]s and others submitted to [inventory] separately.
//! [StubInfo] is instantiated with [pyproject::PyProject] to get where to generate the stub file,
//! and [StubInfo::generate] generates the stub files for every modules.
//!

pub use inventory;
pub use pyo3_stub_gen_derive as derive; // re-export to use in generated code

pub mod generate;
pub mod pyproject;
pub mod type_info;

pub type Result<T> = anyhow::Result<T>;
pub use generate::StubInfo;

#[macro_export]
macro_rules! define_stub_info_gatherer {
    ($function_name:ident) => {
        /// Auto-generated function to gather information to generate stub files
        pub fn $function_name() -> $crate::Result<$crate::StubInfo> {
            let manifest_dir: &::std::path::Path = env!("CARGO_MANIFEST_DIR").as_ref();
            $crate::StubInfo::from_pyproject_toml(manifest_dir.join("pyproject.toml"))
        }
    };
}
