use std::fmt;

/// Normalize a docstring by trimming outer whitespace and dedenting.
///
/// Implements Python's inspect.cleandoc() behavior:
/// 1. Trim leading/trailing whitespace from the entire string
/// 2. Find minimum indentation of non-empty lines (skip first line)
/// 3. Remove that indentation from all lines
///
/// # Examples
/// ```
/// # use pyo3_stub_gen::generate::normalize_docstring;
/// let doc = r#"
///     First line
///     Second line
///         Indented line
/// "#;
/// let normalized = normalize_docstring(doc);
/// assert_eq!(normalized, "First line\nSecond line\n    Indented line");
/// ```
pub fn normalize_docstring(doc: &str) -> String {
    let doc = doc.trim();
    if doc.is_empty() {
        return String::new();
    }

    let lines: Vec<&str> = doc.lines().collect();

    // Find minimum indentation of non-empty lines (skip first line)
    let min_indent = lines
        .iter()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
        .min()
        .unwrap_or(0);

    // Build normalized lines with dedenting applied
    let normalized_lines: Vec<String> = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            if i == 0 {
                // First line: use as-is (already trimmed by outer trim())
                line.to_string()
            } else if line.trim().is_empty() {
                // Empty line: keep it but remove whitespace
                String::new()
            } else {
                // Other lines: remove common indentation
                if line.len() >= min_indent {
                    line[min_indent..].to_string()
                } else {
                    line.trim_start().to_string()
                }
            }
        })
        .collect();

    normalized_lines.join("\n")
}

pub fn write_docstring(f: &mut impl fmt::Write, doc: &str, indent: &str) -> fmt::Result {
    // Docstrings should already be normalized, but trim again for safety
    let doc = doc.trim();
    if !doc.is_empty() {
        let needs_escaping = doc.contains("\"\"\"") || doc.contains(['\0', '\r']);
        let prefix = if needs_escaping { "" } else { "r" };
        writeln!(f, "{indent}{prefix}\"\"\"")?;
        for line in doc.lines() {
            write!(f, "{indent}")?;
            if needs_escaping {
                for ch in line.chars() {
                    match ch {
                        '\\' => f.write_str("\\\\")?,
                        '"' => f.write_str("\\\"")?,
                        '\0' => f.write_str("\\x00")?,
                        '\r' => f.write_str("\\r")?,
                        _ => f.write_char(ch)?,
                    }
                }
            } else {
                f.write_str(line)?;
            }
            writeln!(f)?;
        }
        writeln!(f, r#"{indent}""""#)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::Module;
    use pyo3::{prelude::*, types::PyModule};
    use std::ffi::CString;
    use test_case::test_case;

    #[test_case("First line\n\n    Indented line"; "multiline")]
    #[test_case(r#"Example: """triple quoted""" and '''single quoted'''."#; "triple_quotes")]
    #[test_case(r#"Quoted: \"\"\" and \"""."#; "escaped_quotes")]
    #[test_case(r#"Paths: C:\new\tools\ and escapes: \n \t \x00 \u1234."#; "literal_backslashes")]
    #[test_case(r#"Code: """C:\new\tools""" and \n \u1234."#; "quoted_literal_backslashes")]
    #[test_case("Line ends with \\\nnext line"; "backslash_before_newline")]
    #[test_case("Trailing backslash \\"; "trailing_backslash")]
    #[test_case("Triple quotes \"\"\" and trailing backslash \\"; "escaped_trailing_backslash")]
    #[test_case("Control: \0 and \r inside"; "control_characters")]
    #[test_case("Unicode: 日本語 🦀 with \"\"\" quotes"; "unicode")]
    fn docstrings_round_trip_through_python(doc: &str) {
        let mut source = Module {
            doc: doc.into(),
            ..Default::default()
        }
        .format_init_py();
        source.push_str("\ndef documented():\n");
        write_docstring(&mut source, doc, "    ").unwrap();
        source.push_str("    pass\n");

        Python::initialize();
        Python::attach(|py| {
            let source = CString::new(source).unwrap();
            let module = PyModule::new(py, "docstring_test").unwrap();
            py.run(&source, Some(&module.dict()), None).unwrap();
            let inspect = py.import("inspect").unwrap();
            for object in [
                module.clone().into_any(),
                module.getattr("documented").unwrap(),
            ] {
                let actual: String = inspect
                    .call_method1("getdoc", (object,))
                    .unwrap()
                    .extract()
                    .unwrap();
                assert_eq!(actual, doc);
            }
        });
    }
}
