use crate::{generate::Import, stub_type::ImportRef, type_info::*, TypeInfo};
use std::{collections::HashSet, fmt};

#[derive(Debug, Clone, PartialEq)]
pub struct Arg {
    pub name: &'static str,
    pub r#type: TypeInfo,
    pub signature: Option<SignatureArg>,
}

impl Import for Arg {
    fn import(&self) -> HashSet<ImportRef> {
        self.r#type.import.clone()
    }
}

impl From<&ArgInfo> for Arg {
    fn from(info: &ArgInfo) -> Self {
        Self {
            name: info.name,
            r#type: (info.r#type)(),
            signature: info.signature.clone(),
        }
    }
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(signature) = &self.signature {
            match signature {
                SignatureArg::Ident => write!(f, "{}:{}", self.name, self.r#type),
                SignatureArg::Assign { default } => {
                    write!(f, "{}:{}={}", self.name, self.r#type, default())
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
