//! Context-aware type name qualification for Python stub files.
//!
//! This module provides utilities to qualify type identifiers within compound type expressions
//! based on the target module context. For example, `typing.Optional[ClassA]` should become
//! `typing.Optional[sub_mod.ClassA]` when ClassA is from a different module.

use crate::stub_type::{ImportKind, TypeIdentifierRef};
use std::collections::HashMap;

/// Token types in Python type expressions
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Token {
    /// Bare identifier (e.g., "ClassA", "int")
    Identifier(String),
    /// Dotted path (e.g., "typing.Optional", "collections.abc.Callable")
    DottedPath(Vec<String>),
    /// Opening bracket: [ or (
    OpenBracket(char),
    /// Closing bracket: ] or )
    CloseBracket(char),
    /// Comma separator
    Comma,
    /// Pipe operator for unions (PEP 604)
    Pipe,
    /// Ellipsis (...)
    Ellipsis,
    /// String literal for forward references
    StringLiteral(String),
    /// Whitespace (preserved for formatting)
    Whitespace(String),
    /// Numeric literal (e.g., "42", "3.14", "-1")
    NumericLiteral(String),
}

/// Tokenizes a Python type expression into tokens.
///
/// Handles:
/// - Identifiers: `ClassA`, `int`, `str`
/// - Dotted paths: `typing.Optional`, `collections.abc.Callable`
/// - Brackets: `[`, `]`, `(`, `)`
/// - Special characters: `,`, `|`, `...`
/// - String literals: `"ForwardRef"`
/// - Whitespace preservation
pub(crate) fn tokenize(expr: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = expr.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            // Whitespace
            ' ' | '\t' | '\n' | '\r' => {
                let mut ws = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_whitespace() {
                        ws.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Whitespace(ws));
            }

            // Brackets
            '[' | '(' => {
                tokens.push(Token::OpenBracket(ch));
                chars.next();
            }
            ']' | ')' => {
                tokens.push(Token::CloseBracket(ch));
                chars.next();
            }

            // Comma
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }

            // Pipe (union operator)
            '|' => {
                tokens.push(Token::Pipe);
                chars.next();
            }

            // String literals (forward references)
            '"' | '\'' => {
                let quote_char = ch;
                chars.next(); // consume opening quote
                let mut content = String::new();

                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == quote_char {
                        break;
                    }
                    // Handle escape sequences
                    if c == '\\' {
                        if let Some(&next) = chars.peek() {
                            content.push(c);
                            content.push(next);
                            chars.next();
                        }
                    } else {
                        content.push(c);
                    }
                }

                tokens.push(Token::StringLiteral(content));
            }

            // Dot - could be start of ellipsis or part of dotted path
            '.' => {
                // Look ahead for ellipsis
                let mut peek_chars = chars.clone();
                peek_chars.next(); // skip first dot
                if matches!(peek_chars.peek(), Some(&'.')) {
                    peek_chars.next();
                    if matches!(peek_chars.peek(), Some(&'.')) {
                        // It's an ellipsis
                        chars.next();
                        chars.next();
                        chars.next();
                        tokens.push(Token::Ellipsis);
                        continue;
                    }
                }

                // Otherwise, it's part of a dotted path - this shouldn't happen
                // as dots should be consumed as part of identifiers
                chars.next();
            }

            // Identifier or dotted path
            _ if ch.is_alphabetic() || ch == '_' => {
                let mut ident = String::new();
                let mut parts = Vec::new();

                // Read first identifier
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }

                parts.push(ident.clone());

                // Check for dotted path
                while let Some(&'.') = chars.peek() {
                    // Look ahead to see if there's an identifier after the dot
                    let mut peek = chars.clone();
                    peek.next(); // skip dot

                    if let Some(&c) = peek.peek() {
                        if c.is_alphabetic() || c == '_' {
                            // It's a dotted path
                            chars.next(); // consume dot
                            ident.clear();

                            while let Some(&c) = chars.peek() {
                                if c.is_alphanumeric() || c == '_' {
                                    ident.push(c);
                                    chars.next();
                                } else {
                                    break;
                                }
                            }

                            parts.push(ident.clone());
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                // Create token based on whether it's a dotted path
                if parts.len() > 1 {
                    tokens.push(Token::DottedPath(parts));
                } else {
                    tokens.push(Token::Identifier(parts[0].clone()));
                }
            }

            // Numeric literals (e.g., 42, 3.14, -1)
            _ if ch.is_ascii_digit()
                || (ch == '-' && chars.clone().nth(1).is_some_and(|c| c.is_ascii_digit())) =>
            {
                let mut num = String::new();
                // Handle negative sign
                if ch == '-' {
                    num.push(ch);
                    chars.next();
                }
                // Read digits, dots, and scientific notation
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit()
                        || c == '.'
                        || c == 'e'
                        || c == 'E'
                        || c == '+'
                        || c == '-'
                    {
                        // Special handling: dot must be followed by digit for float
                        if c == '.' {
                            let mut peek = chars.clone();
                            peek.next();
                            if !peek.peek().is_some_and(|&d| d.is_ascii_digit()) {
                                break;
                            }
                        }
                        // For +/- in scientific notation, must be after e/E
                        if (c == '+' || c == '-') && !num.ends_with('e') && !num.ends_with('E') {
                            break;
                        }
                        num.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::NumericLiteral(num));
            }

            // Skip other characters (shouldn't happen in valid type expressions)
            _ => {
                chars.next();
            }
        }
    }

    tokens
}

/// Type expression qualifier that rewrites identifiers based on module context.
pub(crate) struct TypeExpressionQualifier;

impl TypeExpressionQualifier {
    /// Qualify a type expression based on the type references
    ///
    /// This rewrites bare identifiers in the expression to add module qualifiers
    /// when necessary, based on the import context.
    ///
    /// # Parameters
    /// - `expr`: The type expression to qualify
    /// - `type_refs`: Map of type names to their module references
    /// - `target_module`: The module where this type expression will be used
    pub(crate) fn qualify_expression(
        expr: &str,
        type_refs: &HashMap<String, TypeIdentifierRef>,
        target_module: &str,
    ) -> String {
        let tokens = tokenize(expr);
        let mut result = String::new();

        for token in tokens {
            match token {
                Token::Identifier(ref name) => {
                    // Check if this identifier needs qualification
                    if let Some(type_ref) = type_refs.get(name) {
                        match type_ref.import_kind {
                            ImportKind::ByName | ImportKind::SameModule => {
                                // Can use unqualified
                                result.push_str(name);
                            }
                            ImportKind::Module => {
                                // Need to qualify with module component
                                if let Some(module_name) = type_ref.module.get() {
                                    // Check if type is from same module as target
                                    if module_name == target_module {
                                        // Same module - use unqualified name
                                        result.push_str(name);
                                    } else {
                                        // Different module - qualify with last component
                                        let module_component =
                                            module_name.rsplit('.').next().unwrap_or(module_name);
                                        result.push_str(module_component);
                                        result.push('.');
                                        result.push_str(name);
                                    }
                                } else {
                                    // No module info, use as-is
                                    result.push_str(name);
                                }
                            }
                        }
                    } else if Self::is_python_builtin(name) {
                        // Known Python builtin or typing construct - use as-is
                        result.push_str(name);
                    } else {
                        // Unknown identifier - preserve as-is
                        result.push_str(name);
                    }
                }
                Token::DottedPath(parts) => {
                    // Check if this is an over-qualified path (e.g., "my_module.Type" when we're already in "my_module")
                    // If the dotted path starts with a module that matches target_module, strip the module prefix
                    // This handles both 2-part paths (module.Type) and 3+ part paths (module.Class.Member)
                    if parts.len() >= 2 {
                        let module_path = &parts[0];

                        // Check if target_module matches or ends with the module_path
                        // E.g., target="pkg.sub_mod" matches module_path="sub_mod"
                        let is_same_module = module_path == target_module
                            || target_module.ends_with(&format!(".{}", module_path));

                        if is_same_module {
                            // Over-qualified - strip the module prefix and join the rest
                            result.push_str(&parts[1..].join("."));
                        } else {
                            // Different module - keep the full qualification
                            result.push_str(&parts.join("."));
                        }
                    } else {
                        // Single-part path - preserve as-is (shouldn't happen for DottedPath)
                        result.push_str(&parts.join("."));
                    }
                }
                Token::OpenBracket(ch) => result.push(ch),
                Token::CloseBracket(ch) => result.push(ch),
                Token::Comma => result.push(','),
                Token::Pipe => result.push_str(" | "),
                Token::Ellipsis => result.push_str("..."),
                Token::StringLiteral(s) => {
                    // String literals (forward references) - wrap in quotes
                    result.push('"');
                    result.push_str(&s);
                    result.push('"');
                }
                Token::Whitespace(ws) => result.push_str(&ws),
                Token::NumericLiteral(num) => result.push_str(&num),
            }
        }

        result
    }

    /// Check if an identifier is a known Python builtin or typing construct
    fn is_python_builtin(identifier: &str) -> bool {
        matches!(
            identifier,
            // typing module types
            "Any" | "Optional" | "Union" | "List" | "Dict" | "Tuple" | "Set" |
            "Callable" | "Sequence" | "Mapping" | "Iterable" | "Iterator" |
            "Literal" | "TypeVar" | "Generic" | "Protocol" | "TypeAlias" |
            "Final" | "ClassVar" | "Annotated" | "TypeGuard" | "Never" |
            // builtins
            "int" | "str" | "float" | "bool" | "bytes" | "bytearray" |
            "list" | "dict" | "tuple" | "set" | "frozenset" |
            "object" | "type" | "None" | "Ellipsis" |
            "complex" | "slice" | "range" | "memoryview" |
            // Special
            "typing" | "collections" | "abc" | "builtins"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stub_type::ModuleRef;

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("ClassA");
        assert_eq!(tokens, vec![Token::Identifier("ClassA".to_string())]);
    }

    #[test]
    fn test_tokenize_optional() {
        let tokens = tokenize("typing.Optional[ClassA]");
        assert_eq!(
            tokens,
            vec![
                Token::DottedPath(vec!["typing".to_string(), "Optional".to_string()]),
                Token::OpenBracket('['),
                Token::Identifier("ClassA".to_string()),
                Token::CloseBracket(']'),
            ]
        );
    }

    #[test]
    fn test_tokenize_callable() {
        let tokens = tokenize("Callable[[ClassA, str], int]");
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("Callable".to_string()),
                Token::OpenBracket('['),
                Token::OpenBracket('['),
                Token::Identifier("ClassA".to_string()),
                Token::Comma,
                Token::Whitespace(" ".to_string()),
                Token::Identifier("str".to_string()),
                Token::CloseBracket(']'),
                Token::Comma,
                Token::Whitespace(" ".to_string()),
                Token::Identifier("int".to_string()),
                Token::CloseBracket(']'),
            ]
        );
    }

    #[test]
    fn test_tokenize_union() {
        let tokens = tokenize("ClassA | ClassB");
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("ClassA".to_string()),
                Token::Whitespace(" ".to_string()),
                Token::Pipe,
                Token::Whitespace(" ".to_string()),
                Token::Identifier("ClassB".to_string()),
            ]
        );
    }

    #[test]
    fn test_qualify_simple() {
        let mut type_refs = HashMap::new();
        type_refs.insert(
            "ClassA".to_string(),
            TypeIdentifierRef {
                module: ModuleRef::Named("test_package.sub_mod".into()),
                import_kind: ImportKind::Module,
            },
        );

        let result =
            TypeExpressionQualifier::qualify_expression("ClassA", &type_refs, "test_package");
        assert_eq!(result, "sub_mod.ClassA");
    }

    #[test]
    fn test_qualify_optional() {
        let mut type_refs = HashMap::new();
        type_refs.insert(
            "ClassA".to_string(),
            TypeIdentifierRef {
                module: ModuleRef::Named("test_package.sub_mod".into()),
                import_kind: ImportKind::Module,
            },
        );

        let result = TypeExpressionQualifier::qualify_expression(
            "typing.Optional[ClassA]",
            &type_refs,
            "test_package",
        );
        assert_eq!(result, "typing.Optional[sub_mod.ClassA]");
    }

    #[test]
    fn test_qualify_same_module() {
        let mut type_refs = HashMap::new();
        type_refs.insert(
            "ClassA".to_string(),
            TypeIdentifierRef {
                module: ModuleRef::Named("test_package.sub_mod".into()),
                import_kind: ImportKind::SameModule,
            },
        );

        let result = TypeExpressionQualifier::qualify_expression(
            "typing.Optional[ClassA]",
            &type_refs,
            "test_package.sub_mod",
        );
        assert_eq!(result, "typing.Optional[ClassA]");
    }

    #[test]
    fn test_qualify_callable() {
        let mut type_refs = HashMap::new();
        type_refs.insert(
            "ClassA".to_string(),
            TypeIdentifierRef {
                module: ModuleRef::Named("test_package.sub_mod".into()),
                import_kind: ImportKind::Module,
            },
        );
        type_refs.insert(
            "ClassB".to_string(),
            TypeIdentifierRef {
                module: ModuleRef::Named("test_package.other_mod".into()),
                import_kind: ImportKind::Module,
            },
        );

        let result = TypeExpressionQualifier::qualify_expression(
            "collections.abc.Callable[[ClassA, str], ClassB]",
            &type_refs,
            "test_package",
        );
        assert_eq!(
            result,
            "collections.abc.Callable[[sub_mod.ClassA, str], other_mod.ClassB]"
        );
    }

    #[test]
    fn test_qualify_dotted_path_three_parts_same_module() {
        // Test: _core.C.C1 in module "pkg._core" should become C.C1
        let result =
            TypeExpressionQualifier::qualify_expression("_core.C.C1", &HashMap::new(), "pkg._core");
        assert_eq!(result, "C.C1");
    }

    #[test]
    fn test_qualify_dotted_path_three_parts_different_module() {
        // Test: _core.C.C1 in module "pkg.other" should stay _core.C.C1
        let result =
            TypeExpressionQualifier::qualify_expression("_core.C.C1", &HashMap::new(), "pkg.other");
        assert_eq!(result, "_core.C.C1");
    }

    #[test]
    fn test_qualify_dotted_path_two_parts_same_module() {
        // Test: _core.C in module "pkg._core" should become C
        let result =
            TypeExpressionQualifier::qualify_expression("_core.C", &HashMap::new(), "pkg._core");
        assert_eq!(result, "C");
    }

    #[test]
    fn test_tokenize_numeric_literals() {
        // Test integer
        let tokens = tokenize("42");
        assert_eq!(tokens, vec![Token::NumericLiteral("42".to_string())]);

        // Test float
        let tokens = tokenize("3.14");
        assert_eq!(tokens, vec![Token::NumericLiteral("3.14".to_string())]);

        // Test negative integer
        let tokens = tokenize("-1");
        assert_eq!(tokens, vec![Token::NumericLiteral("-1".to_string())]);
    }

    #[test]
    fn test_qualify_numeric_literal_preserved() {
        // Test: numeric literals should be preserved as-is
        let result = TypeExpressionQualifier::qualify_expression("2", &HashMap::new(), "pkg._core");
        assert_eq!(result, "2");

        let result =
            TypeExpressionQualifier::qualify_expression("1.0", &HashMap::new(), "pkg._core");
        assert_eq!(result, "1.0");
    }
}
