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
        }
    };
}

impl_map!(HashMap);
impl_map!(BTreeMap);
