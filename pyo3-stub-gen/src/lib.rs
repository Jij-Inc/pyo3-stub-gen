pub use inventory;
pub use pyo3_stub_gen_derive as derive; // re-export to use in generated code

mod generate;
mod pyproject;
pub mod type_info;

pub type Result<T> = anyhow::Result<T>;
pub use generate::StubInfo;
