mod builtins;
mod collections;
mod pyo3;

#[cfg(feature = "numpy")]
mod numpy;

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
    pub fn none() -> Self {
        Self {
            name: "None".to_string(),
            import: HashSet::new(),
        }
    }

    pub fn any() -> Self {
        Self {
            name: "typing.Any".to_string(),
            import: hashset! { "typing".into() },
        }
    }

    pub fn builtin(name: &str) -> Self {
        Self {
            name: name.to_string(),
            import: HashSet::new(),
        }
    }

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

    #[test]
    fn test() {
        assert_eq!(bool::type_input().name, "bool");
        assert!(bool::type_input().import.is_empty());

        assert_eq!(<&str>::type_input().name, "str");
        assert!(<&str>::type_input().import.is_empty());

        assert_eq!(Vec::<u32>::type_input().name, "typing.Sequence[int]");
        assert_eq!(
            Vec::<u32>::type_input().import,
            hashset! { "typing".into() }
        );

        assert_eq!(Vec::<u32>::type_output().name, "list[int]");
        assert!(Vec::<u32>::type_output().import.is_empty());

        assert_eq!(
            HashMap::<u32, String>::type_input().name,
            "typing.Mapping[int, str]"
        );
        assert_eq!(
            HashMap::<u32, String>::type_input().import,
            hashset! { "typing".into() }
        );

        assert_eq!(HashMap::<u32, String>::type_output().name, "dict[int, str]");
        assert!(HashMap::<u32, String>::type_output().import.is_empty());

        assert_eq!(
            HashMap::<u32, Vec<u32>>::type_input().name,
            "typing.Mapping[int, typing.Sequence[int]]"
        );
        assert_eq!(
            HashMap::<u32, Vec<u32>>::type_input().import,
            hashset! { "typing".into() }
        );

        assert_eq!(
            HashMap::<u32, Vec<u32>>::type_output().name,
            "dict[int, list[int]]"
        );
        assert!(HashMap::<u32, Vec<u32>>::type_output().import.is_empty());
    }
}
