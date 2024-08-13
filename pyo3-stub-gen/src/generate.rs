//! Generate Python typing stub file a.k.a. `*.pyi` file.

mod arg;
mod class;
mod enum_;
mod error;
mod function;
mod member;
mod method;
mod module;
mod new;
mod stub_info;

pub use arg::*;
pub use class::*;
pub use enum_::*;
pub use error::*;
pub use function::*;
pub use member::*;
pub use method::*;
pub use module::*;
pub use new::*;
pub use stub_info::*;

fn indent() -> &'static str {
    "    "
}
