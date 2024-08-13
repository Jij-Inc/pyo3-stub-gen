use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeInfo {
    /// The Python type name.
    pub name: String,

    /// Python modules must be imported in the stub file.
    ///
    /// For example, when `name` is `typing.Sequence[int]`, `import` should contain `typing`.
    /// This makes it possible to use user-defined types in the stub file.
    pub import: HashSet<String>,
}

pub trait PyStubType {
    /// The type to be used in the output signature, i.e. return type of the Python function or methods.
    fn type_output() -> TypeInfo;

    /// The type to be used in the input signature, i.e. the arguments of the Python function or methods.
    ///
    /// This defaults to the output type, but can be overridden for types that are not valid input types.
    fn type_input() -> TypeInfo {
        Self::type_output()
    }
}

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

impl_builtin!(bool, "bool");
impl_builtin!(u8, "int");
impl_builtin!(u16, "int");
impl_builtin!(u32, "int");
impl_builtin!(u64, "int");
impl_builtin!(u128, "int");
impl_builtin!(i8, "int");
impl_builtin!(i16, "int");
impl_builtin!(i32, "int");
impl_builtin!(i64, "int");
impl_builtin!(i128, "int");
impl_builtin!(f32, "float");
impl_builtin!(f64, "float");

impl_builtin!(&str, "str");
impl_builtin!(String, "str");

impl<T: PyStubType> PyStubType for Vec<T> {
    fn type_input() -> TypeInfo {
        let TypeInfo { name, mut import } = T::type_input();
        import.insert("typing".to_string());
        TypeInfo {
            name: format!("typing.Sequence[{}]", name),
            import,
        }
    }
    fn type_output() -> TypeInfo {
        let TypeInfo { name, import } = T::type_output();
        TypeInfo {
            name: format!("list[{}]", name),
            import,
        }
    }
}

macro_rules! impl_map {
    ($map:ident) => {
        impl<Key: PyStubType, Value: PyStubType> PyStubType for $map<Key, Value> {
            fn type_input() -> TypeInfo {
                let TypeInfo {
                    name: key_name,
                    mut import,
                } = Key::type_input();
                let TypeInfo {
                    name: value_name,
                    import: value_import,
                } = Value::type_input();
                import.extend(value_import);
                import.insert("typing".to_string());
                TypeInfo {
                    name: format!("typing.Mapping[{}, {}]", key_name, value_name),
                    import,
                }
            }
            fn type_output() -> TypeInfo {
                let TypeInfo {
                    name: key_name,
                    mut import,
                } = Key::type_output();
                let TypeInfo {
                    name: value_name,
                    import: value_import,
                } = Value::type_output();
                import.extend(value_import);
                TypeInfo {
                    name: format!("dict[{}, {}]", key_name, value_name),
                    import,
                }
            }
        }
    };
}

impl_map!(HashMap);
impl_map!(BTreeMap);

#[cfg(test)]
mod test {
    use super::*;
    use maplit::hashset;

    #[test]
    fn test() {
        assert_eq!(bool::type_input().name, "bool");
        assert!(bool::type_input().import.is_empty());

        assert_eq!(<&str>::type_input().name, "str");
        assert!(<&str>::type_input().import.is_empty());

        assert_eq!(Vec::<u32>::type_input().name, "typing.Sequence[int]");
        assert_eq!(
            Vec::<u32>::type_input().import,
            hashset! { "typing".to_string() }
        );

        assert_eq!(Vec::<u32>::type_output().name, "list[int]");
        assert!(Vec::<u32>::type_output().import.is_empty());

        assert_eq!(
            HashMap::<u32, String>::type_input().name,
            "typing.Mapping[int, str]"
        );
        assert_eq!(
            HashMap::<u32, String>::type_input().import,
            hashset! { "typing".to_string() }
        );

        assert_eq!(HashMap::<u32, String>::type_output().name, "dict[int, str]");
        assert!(HashMap::<u32, String>::type_output().import.is_empty());

        assert_eq!(
            HashMap::<u32, Vec<u32>>::type_input().name,
            "typing.Mapping[int, typing.Sequence[int]]"
        );
        assert_eq!(
            HashMap::<u32, Vec<u32>>::type_input().import,
            hashset! { "typing".to_string() }
        );

        assert_eq!(
            HashMap::<u32, Vec<u32>>::type_output().name,
            "dict[int, list[int]]"
        );
        assert!(HashMap::<u32, Vec<u32>>::type_output().import.is_empty());
    }
}
