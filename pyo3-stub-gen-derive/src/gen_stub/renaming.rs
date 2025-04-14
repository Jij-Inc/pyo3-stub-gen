// The following code is copied from "pyo3-macros-backend". If it would be exported, we could reuse it here!

/// Available renaming rules
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenamingRule {
    CamelCase,
    KebabCase,
    Lowercase,
    PascalCase,
    ScreamingKebabCase,
    ScreamingSnakeCase,
    SnakeCase,
    Uppercase,
}

impl RenamingRule {
    pub fn try_new(name: &str) -> Option<Self> {
        match name {
            "camelCase" => Some(RenamingRule::CamelCase),
            "kebab-case" => Some(RenamingRule::KebabCase),
            "lowercase" => Some(RenamingRule::Lowercase),
            "PascalCase" => Some(RenamingRule::PascalCase),
            "SCREAMING-KEBAB-CASE" => Some(RenamingRule::ScreamingKebabCase),
            "SCREAMING_SNAKE_CASE" => Some(RenamingRule::ScreamingSnakeCase),
            "snake_case" => Some(RenamingRule::SnakeCase),
            "UPPERCASE" => Some(RenamingRule::Uppercase),
            _ => None,
        }
    }
}

impl RenamingRule {
    pub fn apply(self, name: &str) -> String {
        use heck::*;

        match self {
            RenamingRule::CamelCase => name.to_lower_camel_case(),
            RenamingRule::KebabCase => name.to_kebab_case(),
            RenamingRule::Lowercase => name.to_lowercase(),
            RenamingRule::PascalCase => name.to_upper_camel_case(),
            RenamingRule::ScreamingKebabCase => name.to_shouty_kebab_case(),
            RenamingRule::ScreamingSnakeCase => name.to_shouty_snake_case(),
            RenamingRule::SnakeCase => name.to_snake_case(),
            RenamingRule::Uppercase => name.to_uppercase(),
        }
    }
}
