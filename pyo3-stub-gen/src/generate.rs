//! Generate Python typing stub file a.k.a. `*.pyi` file.

mod arg;
mod class;
mod docstring;
mod enum_;
mod error;
mod function;
mod member;
mod method;
mod module;
mod stub_info;
mod variable;

pub use arg::*;
pub use class::*;
pub use enum_::*;
pub use error::*;
pub use function::*;
pub use member::*;
pub use method::*;
pub use module::*;
pub use stub_info::*;
pub use variable::*;

use crate::stub_type::ModuleRef;
use std::collections::HashSet;

fn indent() -> &'static str {
    "    "
}

pub trait Import {
    fn import(&self) -> HashSet<ModuleRef>;
}

pub trait Build {
    type DefType;
    fn build(&self, current_module_name: &str) -> Self::DefType;
}

impl<T> Build for Option<T>
where
    T: Build,
{
    type DefType = Option<T::DefType>;

    fn build(&self, current_module_name: &str) -> Self::DefType {
        self.as_ref().map(|x| x.build(current_module_name))
    }
}

impl Build for crate::type_info::TypeInfoBuilderFcn {
    type DefType = crate::TypeInfo;

    fn build(&self, current_module_name: &str) -> Self::DefType {
        (self)(current_module_name)
    }
}

impl<T> Build for &'static [T]
where
    T: Build,
{
    type DefType = Vec<T::DefType>;

    fn build(&self, current_module_name: &str) -> Self::DefType {
        self.iter().map(|x| x.build(current_module_name)).collect()
    }
}
