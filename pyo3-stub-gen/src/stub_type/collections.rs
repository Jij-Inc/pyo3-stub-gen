use crate::stub_type::*;
use std::collections::{BTreeMap, BTreeSet, HashMap};

impl<T: PyStubType> PyStubType for Option<T> {
    fn type_input(current_module_name: &str) -> TypeInfo {
        let TypeInfo { name, mut import } = T::type_input(current_module_name);
        import.insert("typing".into());
        TypeInfo {
            name: format!("typing.Optional[{}]", name),
            import,
        }
    }
    fn type_output(current_module_name: &str) -> TypeInfo {
        let TypeInfo { name, mut import } = T::type_output(current_module_name);
        import.insert("typing".into());
        TypeInfo {
            name: format!("typing.Optional[{}]", name),
            import,
        }
    }
}

impl<T: PyStubType> PyStubType for Box<T> {
    fn type_input(current_module_name: &str) -> TypeInfo {
        T::type_input(current_module_name)
    }
    fn type_output(current_module_name: &str) -> TypeInfo {
        T::type_output(current_module_name)
    }
}

impl<T: PyStubType, E> PyStubType for Result<T, E> {
    fn type_input(current_module_name: &str) -> TypeInfo {
        T::type_input(current_module_name)
    }
    fn type_output(current_module_name: &str) -> TypeInfo {
        T::type_output(current_module_name)
    }
}

impl<T: PyStubType> PyStubType for Vec<T> {
    fn type_input(current_module_name: &str) -> TypeInfo {
        let TypeInfo { name, mut import } = T::type_input(current_module_name);
        import.insert("typing".into());
        TypeInfo {
            name: format!("typing.Sequence[{}]", name),
            import,
        }
    }
    fn type_output(current_module_name: &str) -> TypeInfo {
        TypeInfo::list_of::<T>(current_module_name)
    }
}

impl<T: PyStubType, const N: usize> PyStubType for [T; N] {
    fn type_input(current_module_name: &str) -> TypeInfo {
        let TypeInfo { name, mut import } = T::type_input(current_module_name);
        import.insert("typing".into());
        TypeInfo {
            name: format!("typing.Sequence[{}]", name),
            import,
        }
    }
    fn type_output(current_module_name: &str) -> TypeInfo {
        TypeInfo::list_of::<T>(current_module_name)
    }
}

impl<T: PyStubType, State> PyStubType for HashSet<T, State> {
    fn type_output(current_module_name: &str) -> TypeInfo {
        TypeInfo::set_of::<T>(current_module_name)
    }
}

impl<T: PyStubType> PyStubType for BTreeSet<T> {
    fn type_output(current_module_name: &str) -> TypeInfo {
        TypeInfo::set_of::<T>(current_module_name)
    }
}

impl<T: PyStubType> PyStubType for indexmap::IndexSet<T> {
    fn type_output(current_module_name: &str) -> TypeInfo {
        TypeInfo::set_of::<T>(current_module_name)
    }
}

macro_rules! impl_map_inner {
    () => {
        fn type_input(current_module_name: &str) -> TypeInfo {
            let TypeInfo {
                name: key_name,
                mut import,
            } = Key::type_input(current_module_name);
            let TypeInfo {
                name: value_name,
                import: value_import,
            } = Value::type_input(current_module_name);
            import.extend(value_import);
            import.insert("typing".into());
            TypeInfo {
                name: format!("typing.Mapping[{}, {}]", key_name, value_name),
                import,
            }
        }
        fn type_output(current_module_name: &str) -> TypeInfo {
            let TypeInfo {
                name: key_name,
                mut import,
            } = Key::type_output(current_module_name);
            let TypeInfo {
                name: value_name,
                import: value_import,
            } = Value::type_output(current_module_name);
            import.extend(value_import);
            import.insert("builtins".into());
            TypeInfo {
                name: format!("builtins.dict[{}, {}]", key_name, value_name),
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

impl<Key: PyStubType, Value: PyStubType, State> PyStubType
    for indexmap::IndexMap<Key, Value, State>
{
    impl_map_inner!();
}

macro_rules! impl_tuple {
    ($($T:ident),*) => {
        impl<$($T: PyStubType),*> PyStubType for ($($T),*) {
            fn type_output(current_module_name: &str) -> TypeInfo {
                let mut merged = HashSet::new();
                let mut names = Vec::new();
                $(
                let TypeInfo { name, import } = $T::type_output(current_module_name);
                names.push(name);
                merged.extend(import);
                )*
                TypeInfo {
                    name: format!("tuple[{}]", names.join(", ")),
                    import: merged,
                }
            }
            fn type_input(current_module_name: &str) -> TypeInfo {
                let mut merged = HashSet::new();
                let mut names = Vec::new();
                $(
                let TypeInfo { name, import } = $T::type_input(current_module_name);
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
