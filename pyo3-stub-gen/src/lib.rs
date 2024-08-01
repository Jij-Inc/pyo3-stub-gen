pub use inventory; // re-export to use in generated code

mod generate;
mod pyproject;
pub mod type_info;

pub type Result<T> = anyhow::Result<T>;
pub use generate::StubInfo;
