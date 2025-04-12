//! Define PyStubType for built-in types based on <https://pyo3.rs/v0.22.2/conversions/tables#argument-types>

use crate::stub_type::*;
use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    path::PathBuf,
    rc::Rc,
    sync::Arc,
    time::SystemTime,
};

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};

macro_rules! impl_builtin {
    ($ty:ty, $pytype:expr) => {
        impl PyStubType for $ty {
            fn type_output() -> TypeInfo {
                TypeInfo::builtin($pytype)
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

// NOTE:
impl PyStubType for () {
    fn type_output() -> TypeInfo {
        TypeInfo::none()
    }
}
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
impl_builtin!(Cow<'_, [u8]>, "bytes");

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
impl_with_module!(chrono::Duration, "datetime.timedelta", "datetime");

impl<T: PyStubType> PyStubType for &T {
    fn type_input() -> TypeInfo {
        T::type_input()
    }
    fn type_output() -> TypeInfo {
        T::type_output()
    }
}

impl<T: PyStubType> PyStubType for Rc<T> {
    fn type_input() -> TypeInfo {
        T::type_input()
    }
    fn type_output() -> TypeInfo {
        T::type_output()
    }
}

impl<T: PyStubType> PyStubType for Arc<T> {
    fn type_input() -> TypeInfo {
        T::type_input()
    }
    fn type_output() -> TypeInfo {
        T::type_output()
    }
}
