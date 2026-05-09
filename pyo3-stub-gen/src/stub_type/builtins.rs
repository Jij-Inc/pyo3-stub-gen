//! Define PyStubType for built-in types based on <https://pyo3.rs/v0.22.2/conversions/tables#argument-types>

use crate::runtime::{union_type, PyRuntimeType};
use crate::stub_type::*;
use ::pyo3::prelude::*;
use ::pyo3::types::{PyBool, PyComplex, PyFloat, PyInt, PyString};
use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    path::PathBuf,
    rc::Rc,
    sync::Arc,
    time::SystemTime,
};

macro_rules! impl_builtin {
    ($ty:ty, $pytype:expr, $py_type_obj:ty) => {
        impl PyStubType for $ty {
            fn type_output() -> TypeInfo {
                TypeInfo::builtin($pytype)
            }
        }
        impl PyRuntimeType for $ty {
            fn runtime_type_object(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
                Ok(py.get_type::<$py_type_obj>().into_any())
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
        impl PyRuntimeType for $ty {
            fn runtime_type_object(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
                // Import the type from the Python module
                let module = py.import($module)?;
                // Extract just the type name from "module.TypeName"
                let type_name = $pytype.rsplit('.').next().unwrap_or($pytype);
                module.getattr(type_name)
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
impl PyRuntimeType for () {
    fn runtime_type_object(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        // None type in Python
        Ok(py.get_type::<::pyo3::types::PyNone>().into_any())
    }
}

impl_builtin!(bool, "bool", PyBool);
impl_builtin!(u8, "int", PyInt);
impl_builtin!(u16, "int", PyInt);
impl_builtin!(u32, "int", PyInt);
impl_builtin!(u64, "int", PyInt);
impl_builtin!(u128, "int", PyInt);
impl_builtin!(usize, "int", PyInt);
impl_builtin!(i8, "int", PyInt);
impl_builtin!(i16, "int", PyInt);
impl_builtin!(i32, "int", PyInt);
impl_builtin!(i64, "int", PyInt);
impl_builtin!(i128, "int", PyInt);
impl_builtin!(isize, "int", PyInt);
impl_builtin!(f32, "float", PyFloat);
impl_builtin!(f64, "float", PyFloat);
impl_builtin!(num_complex::Complex32, "complex", PyComplex);
impl_builtin!(num_complex::Complex64, "complex", PyComplex);

impl_builtin!(char, "str", PyString);
impl_builtin!(&str, "str", PyString);
impl_builtin!(OsStr, "str", PyString);
impl_builtin!(String, "str", PyString);
impl_builtin!(OsString, "str", PyString);
impl_builtin!(Cow<'_, str>, "str", PyString);
impl_builtin!(Cow<'_, OsStr>, "str", PyString);
impl_builtin!(Cow<'_, [u8]>, "bytes", ::pyo3::types::PyBytes);

#[cfg(feature = "ordered-float")]
mod impl_ordered_float {
    use super::*;
    use ::pyo3::types::PyFloat;
    impl_builtin!(ordered_float::NotNan<f32>, "float", PyFloat);
    impl_builtin!(ordered_float::NotNan<f64>, "float", PyFloat);
    impl_builtin!(ordered_float::OrderedFloat<f32>, "float", PyFloat);
    impl_builtin!(ordered_float::OrderedFloat<f64>, "float", PyFloat);
}

impl PyStubType for PathBuf {
    fn type_output() -> TypeInfo {
        TypeInfo::with_module("pathlib.Path", "pathlib".into())
    }
    fn type_input() -> TypeInfo {
        TypeInfo::builtin("str")
            | TypeInfo::with_module("os.PathLike", "os".into())
            | TypeInfo::with_module("pathlib.Path", "pathlib".into())
    }
}
impl PyRuntimeType for PathBuf {
    fn runtime_type_object(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        let pathlib = py.import("pathlib")?;
        pathlib.getattr("Path")
    }
}

impl<Tz: chrono::TimeZone> PyStubType for chrono::DateTime<Tz> {
    fn type_output() -> TypeInfo {
        TypeInfo::with_module("datetime.datetime", "datetime".into())
    }
}
impl<Tz: chrono::TimeZone> PyRuntimeType for chrono::DateTime<Tz> {
    fn runtime_type_object(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        let datetime = py.import("datetime")?;
        datetime.getattr("datetime")
    }
}

impl_with_module!(SystemTime, "datetime.datetime", "datetime");
impl_with_module!(chrono::NaiveDateTime, "datetime.datetime", "datetime");
impl_with_module!(chrono::NaiveDate, "datetime.date", "datetime");
impl_with_module!(chrono::NaiveTime, "datetime.time", "datetime");
impl_with_module!(chrono::FixedOffset, "datetime.tzinfo", "datetime");
impl_with_module!(chrono::Utc, "datetime.tzinfo", "datetime");
impl_with_module!(std::time::Duration, "datetime.timedelta", "datetime");
impl_with_module!(chrono::Duration, "datetime.timedelta", "datetime");
impl_with_module!(time::Duration, "datetime.timedelta", "datetime");
impl_with_module!(time::Date, "datetime.date", "datetime");
impl_with_module!(time::OffsetDateTime, "datetime.datetime", "datetime");
impl_with_module!(time::PrimitiveDateTime, "datetime.datetime", "datetime");
impl_with_module!(time::UtcDateTime, "datetime.datetime", "datetime");
impl_with_module!(time::Time, "datetime.time", "datetime");
impl_with_module!(time::UtcOffset, "datetime.tzinfo", "datetime");
impl_with_module!(std::net::Ipv4Addr, "ipaddress.IPv4Address", "ipaddress");
impl_with_module!(std::net::Ipv6Addr, "ipaddress.IPv6Address", "ipaddress");

impl PyStubType for std::net::IpAddr {
    fn type_output() -> TypeInfo {
        TypeInfo::with_module(
            "ipaddress.IPv4Address",
            ModuleRef::Named("ipaddress".to_string()),
        ) | TypeInfo::with_module(
            "ipaddress.IPv6Address",
            ModuleRef::Named("ipaddress".to_string()),
        )
    }
}
impl PyRuntimeType for std::net::IpAddr {
    fn runtime_type_object(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        let module = py.import("ipaddress")?;
        let type_v4 = module.getattr("IPv4Address")?;
        let type_v6 = module.getattr("IPv6Address")?;
        union_type(py, &[type_v4, type_v6])
    }
}

impl<T: PyStubType> PyStubType for &T {
    fn type_input() -> TypeInfo {
        T::type_input()
    }
    fn type_output() -> TypeInfo {
        T::type_output()
    }
}
impl<T: PyRuntimeType> PyRuntimeType for &T {
    fn runtime_type_object(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        T::runtime_type_object(py)
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
impl<T: PyRuntimeType> PyRuntimeType for Rc<T> {
    fn runtime_type_object(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        T::runtime_type_object(py)
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
impl<T: PyRuntimeType> PyRuntimeType for Arc<T> {
    fn runtime_type_object(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        T::runtime_type_object(py)
    }
}
