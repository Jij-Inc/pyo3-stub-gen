use super::{PyStubType, TypeInfo, ImportRef};
use maplit::hashset;
use numpy::{
    ndarray::Dimension, Element, PyArray, PyArrayDescr, PyReadonlyArray, PyReadwriteArray,
    PyUntypedArray,
};

trait NumPyScalar {
    fn type_() -> TypeInfo;
}

macro_rules! impl_numpy_scalar {
    ($ty:ty, $name:expr) => {
        impl NumPyScalar for $ty {
            fn type_() -> TypeInfo {
                TypeInfo {
                    name: format!("numpy.{}", $name),
                    import: hashset![ImportRef::Module("numpy".into())],
                }
            }
        }
    };
}

impl_numpy_scalar!(i8, "int8");
impl_numpy_scalar!(i16, "int16");
impl_numpy_scalar!(i32, "int32");
impl_numpy_scalar!(i64, "int64");
impl_numpy_scalar!(u8, "uint8");
impl_numpy_scalar!(u16, "uint16");
impl_numpy_scalar!(u32, "uint32");
impl_numpy_scalar!(u64, "uint64");
impl_numpy_scalar!(f32, "float32");
impl_numpy_scalar!(f64, "float64");
impl_numpy_scalar!(num_complex::Complex32, "complex64");
impl_numpy_scalar!(num_complex::Complex64, "complex128");

impl<T: NumPyScalar, D> PyStubType for PyArray<T, D> {
    fn type_output() -> TypeInfo {
        let TypeInfo { name, mut import } = T::type_();
        import.insert(ImportRef::Module("numpy.typing".into()));
        TypeInfo {
            name: format!("numpy.typing.NDArray[{name}]"),
            import,
        }
    }
}

impl PyStubType for PyUntypedArray {
    fn type_output() -> TypeInfo {
        TypeInfo {
            name: "numpy.typing.NDArray[typing.Any]".into(),
            import: hashset![ImportRef::Module("numpy.typing".into()), ImportRef::Module("typing".into())],
        }
    }
}

impl<T, D> PyStubType for PyReadonlyArray<'_, T, D>
where
    T: NumPyScalar + Element,
    D: Dimension,
{
    fn type_output() -> TypeInfo {
        PyArray::<T, D>::type_output()
    }
}

impl<T, D> PyStubType for PyReadwriteArray<'_, T, D>
where
    T: NumPyScalar + Element,
    D: Dimension,
{
    fn type_output() -> TypeInfo {
        PyArray::<T, D>::type_output()
    }
}

impl PyStubType for PyArrayDescr {
    fn type_output() -> TypeInfo {
        TypeInfo {
            name: "numpy.dtype".into(),
            import: hashset![ImportRef::Module("numpy".into())],
        }
    }
}
