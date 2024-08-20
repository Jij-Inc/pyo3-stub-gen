use crate::stub_type::*;
use std::collections::{BTreeMap, HashMap};

impl<T: PyStubType> PyStubType for Option<T> {
    fn type_input() -> TypeInfo {
        let TypeInfo { name, mut import } = T::type_input();
        import.insert("typing".into());
        TypeInfo {
            name: format!("typing.Optional[{}]", name),
            import,
        }
    }
    fn type_output() -> TypeInfo {
        let TypeInfo { name, mut import } = T::type_output();
        import.insert("typing".into());
        TypeInfo {
            name: format!("typing.Optional[{}]", name),
            import,
        }
    }
}

impl<T: PyStubType> PyStubType for Box<T> {
    fn type_input() -> TypeInfo {
        T::type_input()
    }
    fn type_output() -> TypeInfo {
        T::type_output()
    }
}

impl<T: PyStubType, E> PyStubType for Result<T, E> {
    fn type_input() -> TypeInfo {
        T::type_input()
    }
    fn type_output() -> TypeInfo {
        T::type_output()
    }
}

impl<T: PyStubType> PyStubType for Vec<T> {
    fn type_input() -> TypeInfo {
        let TypeInfo { name, mut import } = T::type_input();
        import.insert("typing".into());
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

macro_rules! impl_map_inner {
    () => {
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
            import.insert("typing".into());
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
    };
}

impl<Key: PyStubType, Value: PyStubType> PyStubType for BTreeMap<Key, Value> {
    impl_map_inner!();
}

impl<Key: PyStubType, Value: PyStubType, State> PyStubType for HashMap<Key, Value, State> {
    impl_map_inner!();
}

macro_rules! impl_tuple {
    ($($T:ident),*) => {
        impl<$($T: PyStubType),*> PyStubType for ($($T),*) {
            fn type_output() -> TypeInfo {
                let mut merged = HashSet::new();
                let mut names = Vec::new();
                $(
                let TypeInfo { name, import } = $T::type_output();
                names.push(name);
                merged.extend(import);
                )*
                TypeInfo {
                    name: format!("tuple[{}]", names.join(", ")),
                    import: merged,
                }
            }
            fn type_input() -> TypeInfo {
                let mut merged = HashSet::new();
                let mut names = Vec::new();
                $(
                let TypeInfo { name, import } = $T::type_input();
                names.push(name);
                merged.extend(import);
                )*
                TypeInfo {
                    name: format!("tuple[{}]", names.join(", ")),
                    import: merged,
                }
            }
        }
    };
}

impl_tuple!(T1, T2);
impl_tuple!(T1, T2, T3);
impl_tuple!(T1, T2, T3, T4);
impl_tuple!(T1, T2, T3, T4, T5);
impl_tuple!(T1, T2, T3, T4, T5, T6);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
