mod builtins;
mod collections;

use std::collections::HashSet;

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

#[cfg(test)]
mod test {
    use super::*;
    use maplit::hashset;
    use std::collections::HashMap;

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
