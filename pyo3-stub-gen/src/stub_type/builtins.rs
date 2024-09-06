//! Define PyStubType for built-in types based on <https://pyo3.rs/v0.22.2/conversions/tables#argument-types>

use crate::stub_type::*;
use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    path::PathBuf,
};

macro_rules! impl_builtin {
    ($ty:ty, $pytype:expr) => {
        impl PyStubType for $ty {
            fn type_output() -> TypeInfo {
                TypeInfo {
                    name: $pytype.to_string(),
                    import: HashSet::new(),
                }
            }
        }
    };
}

macro_rules! impl_with_module {
    ($ty:ty, $pytype:expr, $module:expr) => {
        impl PyStubType for $ty {
            fn type_output() -> TypeInfo {
                TypeInfo::with_module($pytype, $module.into())
            }
        }
    };
}

impl_builtin!((), "None");
impl_builtin!(bool, "bool");
impl_builtin!(u8, "int");
impl_builtin!(u16, "int");
impl_builtin!(u32, "int");
impl_builtin!(u64, "int");
impl_builtin!(u128, "int");
impl_builtin!(usize, "int");
impl_builtin!(i8, "int");
impl_builtin!(i16, "int");
impl_builtin!(i32, "int");
impl_builtin!(i64, "int");
impl_builtin!(i128, "int");
impl_builtin!(isize, "int");
impl_builtin!(f32, "float");
impl_builtin!(f64, "float");
impl_builtin!(num_complex::Complex32, "complex");
impl_builtin!(num_complex::Complex64, "complex");

impl_builtin!(char, "str");
impl_builtin!(&str, "str");
impl_builtin!(OsStr, "str");
impl_builtin!(String, "str");
impl_builtin!(OsString, "str");
impl_builtin!(Cow<'_, str>, "str");
impl_builtin!(Cow<'_, OsStr>, "str");

impl PyStubType for PathBuf {
    fn type_output() -> TypeInfo {
        TypeInfo::builtin("str")
    }
    fn type_input() -> TypeInfo {
        TypeInfo::builtin("str")
            | TypeInfo::with_module("os.PathLike", "os".into())
            | TypeInfo::with_module("pathlib.Path", "pathlib".into())
    }
}

/// Adding chrono types supported by pyo3 [here](https://pyo3.rs/v0.22.2/conversions/tables)
#[cfg(feature = "chrono")]
mod chrono_exports {
    use std::time::SystemTime;

    use super::{PyStubType, TypeInfo};
    use chrono::{
        DateTime, Duration, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc,
    };

    impl<Tz: TimeZone> PyStubType for DateTime<Tz> {
        fn type_output() -> TypeInfo {
            TypeInfo::with_module("datetime.datetime", "datetime".into())
        }
    }

    impl_with_module!(SystemTime, "datetime.datetime", "datetime");
    impl_with_module!(NaiveDateTime, "datetime.datetime", "datetime");
    impl_with_module!(NaiveDate, "datetime.date", "datetime");
    impl_with_module!(NaiveTime, "datetime.time", "datetime");
    impl_with_module!(FixedOffset, "datetime.tzinfo", "datetime");
    impl_with_module!(Utc, "datetime.tzinfo", "datetime");
    impl_with_module!(std::time::Duration, "datetime.timedelta", "datetime");
    impl_with_module!(Duration, "datetime.timedelta", "datetime");
}

#[allow(unused_imports)]
#[cfg(feature = "chrono")]
pub use chrono_exports::*;
