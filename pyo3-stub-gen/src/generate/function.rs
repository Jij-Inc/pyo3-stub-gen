use crate::{generate::*, type_info::*, TypeInfo};
use std::{collections::HashMap, fmt};

/// Definition of a Python function.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub name: &'static str,
    pub args: Vec<Arg>,
    pub r#return: TypeInfo,
    pub signature: Option<&'static str>,
    pub doc: &'static str,
}

impl Import for FunctionDef {
    fn import(&self) -> HashSet<ModuleRef> {
        let mut import = self.r#return.import.clone();
        for arg in &self.args {
            import.extend(arg.import().into_iter());
        }
        import
    }
}

impl From<&PyFunctionInfo> for FunctionDef {
    fn from(info: &PyFunctionInfo) -> Self {
        Self {
            name: info.name,
            args: info.args.iter().map(Arg::from).collect(),
            r#return: (info.r#return)(),
            doc: info.doc,
            signature: info.signature,
        }
    }
}

impl fmt::Display for FunctionDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "def {}(", self.name)?;
        if let Some(sig) = self.signature {
            let mut signature_map = HashMap::new();
            let split_sig = sig.split(",").map(|s| s.trim());
            for s in split_sig {
                let name_value: Vec<&str> = s.split('=').map(|s| s.trim()).collect();
                if name_value.len() == 2 {
                    signature_map.insert(name_value[0].to_string(), name_value[1].to_string());
                } else {
                    continue;
                }
            }
            for (i, arg) in self.args.iter().enumerate() {
                if let Some(value) = signature_map.get(arg.name) {
                    write!(f, "{} = {}", arg, value)?;
                } else {
                    write!(f, "{}", arg)?;
                }
                if i != self.args.len() - 1 {
                    write!(f, ", ")?;
                }
            }
        } else {
            for (i, arg) in self.args.iter().enumerate() {
                write!(f, "{}", arg)?;
                if i != self.args.len() - 1 {
                    write!(f, ", ")?;
                }
            }
        }
        writeln!(f, ") -> {}:", self.r#return)?;

        let doc = self.doc;
        let indent = indent();
        if !doc.is_empty() {
            writeln!(f, r#"{indent}r""""#)?;
            for line in doc.lines() {
                writeln!(f, "{indent}{}", line)?;
            }
            writeln!(f, r#"{indent}""""#)?;
        }
        writeln!(f, "{indent}...")?;
        writeln!(f)?;
        Ok(())
    }
}
