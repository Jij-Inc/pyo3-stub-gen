mod builtins;
mod collections;
mod pyo3;

#[cfg(feature = "numpy")]
mod numpy;

#[cfg(feature = "either")]
mod either;

use maplit::hashset;
use std::{collections::HashSet, fmt, ops};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub enum ModuleRef {
    Named(String),

    /// Default module that PyO3 creates.
    ///
    /// - For pure Rust project, the default module name is the crate name specified in `Cargo.toml`
    ///   or `project.name` specified in `pyproject.toml`
    /// - For mixed Rust/Python project, the default module name is `tool.maturin.module-name` specified in `pyproject.toml`
    ///
    /// Because the default module name cannot be known at compile time, it will be resolved at the time of the stub file generation.
    /// This is a placeholder for the default module name.
    #[default]
    Default,
}

impl ModuleRef {
    pub fn get(&self) -> Option<&str> {
        match self {
            Self::Named(name) => Some(name),
            Self::Default => None,
        }
    }
}

impl From<&str> for ModuleRef {
    fn from(s: &str) -> Self {
        Self::Named(s.to_string())
    }
}

/// Type information for creating Python stub files annotated by [PyStubType] trait.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeInfo {
    /// The Python type name.
    pub name: String,

    /// Python modules must be imported in the stub file.
    ///
    /// For example, when `name` is `typing.Sequence[int]`, `import` should contain `typing`.
    /// This makes it possible to use user-defined types in the stub file.
    pub import: HashSet<ModuleRef>,
}

impl fmt::Display for TypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl TypeInfo {
    /// A `None` type annotation.
    pub fn none() -> Self {
        // NOTE: since 3.10, NoneType is provided from types module,
        // but there is no corresponding definitions prior to 3.10.
        Self {
            name: "None".to_string(),
            import: HashSet::new(),
        }
    }

    /// A `typing.Any` type annotation.
    pub fn any() -> Self {
        Self {
            name: "typing.Any".to_string(),
            import: hashset! { "typing".into() },
        }
    }

    /// A `list[Type]` type annotation.
    pub fn list_of<T: PyStubType>() -> Self {
        let TypeInfo { name, mut import } = T::type_output();
        import.insert("builtins".into());
        TypeInfo {
            name: format!("builtins.list[{name}]"),
            import,
        }
    }

    /// A `set[Type]` type annotation.
    pub fn set_of<T: PyStubType>() -> Self {
        let TypeInfo { name, mut import } = T::type_output();
        import.insert("builtins".into());
        TypeInfo {
            name: format!("builtins.set[{name}]"),
            import,
        }
    }

    /// A `dict[Type]` type annotation.
    pub fn dict_of<K: PyStubType, V: PyStubType>() -> Self {
        let TypeInfo {
            name: name_k,
            mut import,
        } = K::type_output();
        let TypeInfo {
            name: name_v,
            import: import_v,
        } = V::type_output();
        import.extend(import_v);
        import.insert("builtins".into());
        TypeInfo {
            name: format!("builtins.set[{name_k}, {name_v}]"),
            import,
        }
    }

    /// A type annotation of a built-in type provided from `builtins` module, such as `int`, `str`, or `float`. Generic builtin types are also possible, such as `dict[str, str]`.
    pub fn builtin(name: &str) -> Self {
        Self {
            name: format!("builtins.{name}"),
            import: hashset! { "builtins".into() },
        }
    }

    /// Unqualified type.
    pub fn unqualified(name: &str) -> Self {
        Self {
            name: name.to_string(),
            import: hashset! {},
        }
    }

    /// A type annotation of a type that must be imported. The type name must be qualified with the module name:
    ///
    /// ```
    /// pyo3_stub_gen::TypeInfo::with_module("pathlib.Path", "pathlib".into());
    /// ```
    pub fn with_module(name: &str, module: ModuleRef) -> Self {
        let mut import = HashSet::new();
        import.insert(module);
        Self {
            name: name.to_string(),
            import,
        }
    }
}

impl ops::BitOr for TypeInfo {
    type Output = Self;

    fn bitor(mut self, rhs: Self) -> Self {
        self.import.extend(rhs.import);
        Self {
            name: format!("{} | {}", self.name, rhs.name),
            import: self.import,
        }
    }
}

/// Implement [PyStubType]
///
/// ```rust
/// use pyo3::*;
/// use pyo3_stub_gen::{impl_stub_type, derive::*};
///
/// #[gen_stub_pyclass]
/// #[pyclass]
/// struct A;
///
/// #[gen_stub_pyclass]
/// #[pyclass]
/// struct B;
///
/// enum E {
///     A(A),
///     B(B),
/// }
/// impl_stub_type!(E = A | B);
///
/// struct X(A);
/// impl_stub_type!(X = A);
///
/// struct Y {
///    a: A,
///    b: B,
/// }
/// impl_stub_type!(Y = (A, B));
/// ```
#[macro_export]
macro_rules! impl_stub_type {
    ($ty: ty = $($base:ty)|+) => {
        impl ::pyo3_stub_gen::PyStubType for $ty {
            fn type_output() -> ::pyo3_stub_gen::TypeInfo {
                $(<$base>::type_output()) | *
            }
            fn type_input() -> ::pyo3_stub_gen::TypeInfo {
                $(<$base>::type_input()) | *
            }
        }
    };
    ($ty:ty = $base:ty) => {
        impl ::pyo3_stub_gen::PyStubType for $ty {
            fn type_output() -> ::pyo3_stub_gen::TypeInfo {
                <$base>::type_output()
            }
            fn type_input() -> ::pyo3_stub_gen::TypeInfo {
                <$base>::type_input()
            }
        }
    };
}

/// Annotate Rust types with Python type information.
pub trait PyStubType {
    /// The type to be used in the output signature, i.e. return type of the Python function or methods.
    fn type_output() -> TypeInfo;

    /// The type to be used in the input signature, i.e. the arguments of the Python function or methods.
    ///
    /// This defaults to the output type, but can be overridden for types that are not valid input types.
    /// For example, `Vec::<T>::type_output` returns `list[T]` while `Vec::<T>::type_input` returns `typing.Sequence[T]`.
    fn type_input() -> TypeInfo {
        Self::type_output()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use maplit::hashset;
    use std::collections::HashMap;
    use test_case::test_case;

    #[test_case(bool::type_input(), "builtins.bool", hashset! { "builtins".into() } ; "bool_input")]
    #[test_case(<&str>::type_input(), "builtins.str", hashset! { "builtins".into() } ; "str_input")]
    #[test_case(Vec::<u32>::type_input(), "typing.Sequence[builtins.int]", hashset! { "typing".into(), "builtins".into() } ; "Vec_u32_input")]
    #[test_case(Vec::<u32>::type_output(), "builtins.list[builtins.int]", hashset! {  "builtins".into() } ; "Vec_u32_output")]
    #[test_case(HashMap::<u32, String>::type_input(), "typing.Mapping[builtins.int, builtins.str]", hashset! { "typing".into(), "builtins".into() } ; "HashMap_u32_String_input")]
    #[test_case(HashMap::<u32, String>::type_output(), "builtins.dict[builtins.int, builtins.str]", hashset! { "builtins".into() } ; "HashMap_u32_String_output")]
    #[test_case(indexmap::IndexMap::<u32, String>::type_input(), "typing.Mapping[builtins.int, builtins.str]", hashset! { "typing".into(), "builtins".into() } ; "IndexMap_u32_String_input")]
    #[test_case(indexmap::IndexMap::<u32, String>::type_output(), "builtins.dict[builtins.int, builtins.str]", hashset! { "builtins".into() } ; "IndexMap_u32_String_output")]
    #[test_case(HashMap::<u32, Vec<u32>>::type_input(), "typing.Mapping[builtins.int, typing.Sequence[builtins.int]]", hashset! { "builtins".into(), "typing".into() } ; "HashMap_u32_Vec_u32_input")]
    #[test_case(HashMap::<u32, Vec<u32>>::type_output(), "builtins.dict[builtins.int, builtins.list[builtins.int]]", hashset! { "builtins".into() } ; "HashMap_u32_Vec_u32_output")]
    #[test_case(HashSet::<u32>::type_input(), "builtins.set[builtins.int]", hashset! { "builtins".into() } ; "HashSet_u32_input")]
    #[test_case(indexmap::IndexSet::<u32>::type_input(), "builtins.set[builtins.int]", hashset! { "builtins".into() } ; "IndexSet_u32_input")]
    fn test(tinfo: TypeInfo, name: &str, import: HashSet<ModuleRef>) {
        assert_eq!(tinfo.name, name);
        if import.is_empty() {
            assert!(tinfo.import.is_empty());
        } else {
            assert_eq!(tinfo.import, import);
        }
    }
}
