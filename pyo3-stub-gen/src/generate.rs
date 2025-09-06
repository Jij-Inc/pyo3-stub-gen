//! Generate Python typing stub file a.k.a. `*.pyi` file.

mod arg;
mod class;
mod deprecated;
mod docstring;
mod enum_;
mod function;
mod member;
mod method;
mod module;
mod stub_info;
mod variable;
mod variant_methods;

pub use arg::*;
pub use class::*;
pub use enum_::*;
pub use function::*;
pub use member::*;
pub use method::*;
pub use module::*;
pub use stub_info::*;
pub use variable::*;

use crate::stub_type::ImportRef;
use std::collections::HashSet;

fn indent() -> &'static str {
    "    "
}

pub trait Import {
    fn import(&self) -> HashSet<ImportRef>;
}
