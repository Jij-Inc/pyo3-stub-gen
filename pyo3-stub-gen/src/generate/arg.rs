use crate::{generate::*, stub_type::ModuleRef, type_info::*, TypeInfo};
use std::{collections::HashSet, fmt};

#[derive(Debug, Clone, PartialEq)]
pub struct Arg {
    pub name: &'static str,
    pub r#type: TypeInfo,
    pub signature: Option<SignatureArg>,
}

impl Import for Arg {
    fn import(&self) -> HashSet<ModuleRef> {
        self.r#type.import.clone()
    }
}

impl Build for ArgInfo {
    type DefType = Arg;

    fn build(&self, current_module_name: &str) -> Self::DefType {
        Self::DefType {
            name: self.name,
            r#type: self.r#type.build(current_module_name),
            signature: self.signature.clone(),
        }
    }
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(signature) = &self.signature {
            match signature {
                SignatureArg::Ident => write!(f, "{}:{}", self.name, self.r#type),
                SignatureArg::Assign { default } => {
                    let default: &String = default;
                    write!(f, "{}:{}={}", self.name, self.r#type, default)
                }
                SignatureArg::Star => write!(f, "*"),
                SignatureArg::Args => write!(f, "*{}", self.name),
                SignatureArg::Keywords => write!(f, "**{}", self.name),
            }
        } else {
            write!(f, "{}:{}", self.name, self.r#type)
        }
    }
}
