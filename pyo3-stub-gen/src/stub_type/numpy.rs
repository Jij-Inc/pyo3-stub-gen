use super::{PyStubType, TypeInfo};
use numpy::PyArray;

impl<T: PyStubType, D> PyStubType for PyArray<T, D> {
    fn type_output() -> TypeInfo {
        let TypeInfo { name, mut import } = T::type_output();
        import.insert("numpy.typing".into());
        TypeInfo {
            name: format!("numpy.typing.NDArray[{name}]"),
            import,
        }
    }

    fn type_input() -> TypeInfo {
        let TypeInfo { name, mut import } = T::type_input();
        import.insert("numpy.typing".into());
        TypeInfo {
            name: format!("numpy.typing.NDArray[{name}]"),
            import,
        }
    }
}
