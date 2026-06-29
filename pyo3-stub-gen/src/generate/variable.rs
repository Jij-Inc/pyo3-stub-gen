use std::{collections::HashSet, fmt};

use crate::{
    generate::Import, stub_type::ImportRef, type_info::PyTypeVarInfo, type_info::PyVariableInfo,
    TypeInfo,
};

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDef {
    pub name: &'static str,
    pub type_: TypeInfo,
    pub default: Option<String>,
}

impl From<&PyVariableInfo> for VariableDef {
    fn from(info: &PyVariableInfo) -> Self {
        Self {
            name: info.name,
            type_: (info.r#type)(),
            default: info.default.map(|f| f()),
        }
    }
}

impl Import for VariableDef {
    fn import(&self) -> HashSet<ImportRef> {
        self.type_.import.clone()
    }
}

impl fmt::Display for VariableDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.type_)?;
        if let Some(default) = &self.default {
            write!(f, " = {default}")?;
        }
        Ok(())
    }
}

impl VariableDef {
    /// Format variable with module-qualified type names
    ///
    /// This method uses the target module context to qualify type identifiers
    /// within compound type expressions based on their source modules.
    pub fn fmt_for_module(&self, target_module: &str, f: &mut fmt::Formatter) -> fmt::Result {
        let qualified_type = self.type_.qualified_for_module(target_module);
        write!(f, "{}: {}", self.name, qualified_type)?;
        if let Some(default) = &self.default {
            write!(f, " = {default}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeVarDef {
    pub name: &'static str,
    pub constraints: Vec<String>,
    pub bound: Option<String>,
    pub covariant: bool,
    pub contravariant: bool,
}

impl From<&PyTypeVarInfo> for TypeVarDef {
    fn from(info: &PyTypeVarInfo) -> Self {
        Self {
            name: info.name,
            constraints: info.constraints.iter().map(|s| s.to_string()).collect(),
            bound: info.bound.map(|s| s.to_string()),
            covariant: info.covariant,
            contravariant: info.contravariant,
        }
    }
}

impl fmt::Display for TypeVarDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} = typing.TypeVar(\"{}\"", self.name, self.name)?;

        // Add constraints as positional arguments
        for constraint in &self.constraints {
            write!(f, ", {}", constraint)?;
        }

        // Add keyword arguments
        if let Some(bound) = &self.bound {
            write!(f, ", bound={}", bound)?;
        }
        if self.covariant {
            write!(f, ", covariant=True")?;
        }
        if self.contravariant {
            write!(f, ", contravariant=True")?;
        }

        write!(f, ")")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typevar_def_display_basic() {
        let typevar = TypeVarDef {
            name: "T",
            constraints: vec![],
            bound: None,
            covariant: false,
            contravariant: false,
        };
        assert_eq!(typevar.to_string(), "T = typing.TypeVar(\"T\")");
    }

    #[test]
    fn test_typevar_def_with_constraints() {
        let typevar = TypeVarDef {
            name: "T",
            constraints: vec!["str".to_string(), "int".to_string()],
            bound: None,
            covariant: false,
            contravariant: false,
        };
        assert_eq!(typevar.to_string(), "T = typing.TypeVar(\"T\", str, int)");
    }

    #[test]
    fn test_typevar_def_with_bound() {
        let typevar = TypeVarDef {
            name: "T",
            constraints: vec![],
            bound: Some("BaseClass".to_string()),
            covariant: false,
            contravariant: false,
        };
        assert_eq!(
            typevar.to_string(),
            "T = typing.TypeVar(\"T\", bound=BaseClass)"
        );
    }

    #[test]
    fn test_typevar_def_covariant() {
        let typevar = TypeVarDef {
            name: "T",
            constraints: vec![],
            bound: None,
            covariant: true,
            contravariant: false,
        };
        assert_eq!(
            typevar.to_string(),
            "T = typing.TypeVar(\"T\", covariant=True)"
        );
    }

    #[test]
    fn test_typevar_def_contravariant() {
        let typevar = TypeVarDef {
            name: "T",
            constraints: vec![],
            bound: None,
            covariant: false,
            contravariant: true,
        };
        assert_eq!(
            typevar.to_string(),
            "T = typing.TypeVar(\"T\", contravariant=True)"
        );
    }

    #[test]
    fn test_typevar_def_all_parameters() {
        let typevar = TypeVarDef {
            name: "T",
            constraints: vec!["str".to_string(), "int".to_string()],
            bound: None,
            covariant: true,
            contravariant: false,
        };
        assert_eq!(
            typevar.to_string(),
            "T = typing.TypeVar(\"T\", str, int, covariant=True)"
        );
    }

    #[test]
    fn test_typevar_def_from_py_typevar_info() {
        let info = PyTypeVarInfo {
            name: "U",
            module: "test_module",
            constraints: &["str", "int"],
            bound: Some("BaseClass"),
            covariant: true,
            contravariant: false,
        };
        let typevar = TypeVarDef::from(&info);
        assert_eq!(typevar.name, "U");
        assert_eq!(
            typevar.to_string(),
            "U = typing.TypeVar(\"U\", str, int, bound=BaseClass, covariant=True)"
        );
    }
}
