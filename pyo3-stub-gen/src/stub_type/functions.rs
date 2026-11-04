use super::collections::build_type_refs_from_inner;
use super::{PyStubType, TypeInfo};
use crate::runtime::PyRuntimeType;
use ::pyo3::{Bound, PyAny, PyResult, Python};

macro_rules! impl_tuple {
    ($($T:ident),*) => {
        impl<R: PyStubType, $($T: PyStubType),*> PyStubType for fn ($($T),*) -> R {
            fn type_output() -> TypeInfo {
                let info_result = R::type_output();
                let mut type_refs = build_type_refs_from_inner(&info_result);
                let mut merged = info_result.import;
                let mut names = Vec::new();
                $(
                    let info = $T::type_input();
                    type_refs.extend(build_type_refs_from_inner(&info));
                    names.push(info.name);
                    merged.extend(info.import);
                )*
                TypeInfo {
                    name: format!("typing.Callable[[{}], {}]", names.join(", "), info_result.name),
                    source_module: None,
                    import: merged,
                    type_refs,
                }
            }
        }

        impl<R, $($T),*> PyRuntimeType for fn ($($T),*) -> R {
            fn runtime_type_object(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
                Ok(py.get_type::<::pyo3::types::PyAny>().into_any())
            }
        }
    }
}

impl_tuple!(T1);
impl_tuple!(T1, T2);
impl_tuple!(T1, T2, T3);
impl_tuple!(T1, T2, T3, T4);
impl_tuple!(T1, T2, T3, T4, T5);
impl_tuple!(T1, T2, T3, T4, T5, T6);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
