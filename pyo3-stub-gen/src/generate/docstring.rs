use std::fmt;

pub fn write_docstring(f: &mut fmt::Formatter, doc: &str, indent: &str) -> fmt::Result {
    let doc = doc.trim();
    if !doc.is_empty() {
        writeln!(f, r#"{indent}r""""#)?;
        for line in doc.lines() {
            writeln!(f, "{indent}{line}")?;
        }
        writeln!(f, r#"{indent}""""#)?;
    }
    Ok(())
}
