use std::fmt;

pub fn write_docstring(f: &mut fmt::Formatter, doc: &str, indent: &str) -> fmt::Result {
    let doc = doc.trim();
    if !doc.is_empty() {
        writeln!(f, r#"{indent}r""""#)?;

        // Dedent the docstring (similar to Python's textwrap.dedent)
        let lines: Vec<&str> = doc.lines().collect();

        // Find the minimum indentation of non-empty lines (excluding the first line)
        let min_indent = lines
            .iter()
            .skip(1) // Skip first line as it's usually right after the opening """
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
            .min()
            .unwrap_or(0);

        // Write each line with common indentation removed
        for (i, line) in lines.iter().enumerate() {
            if i == 0 {
                // First line: write as-is (it's usually not indented in the original)
                writeln!(f, "{indent}{line}")?;
            } else if line.trim().is_empty() {
                // Empty line: write just the base indent
                writeln!(f, "{indent}")?;
            } else {
                // Other lines: remove common indentation
                let dedented = if line.len() >= min_indent {
                    &line[min_indent..]
                } else {
                    line.trim_start()
                };
                writeln!(f, "{indent}{dedented}")?;
            }
        }

        writeln!(f, r#"{indent}""""#)?;
    }
    Ok(())
}
