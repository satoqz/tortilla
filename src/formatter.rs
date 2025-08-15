use std::io;

use crate::parser::Paragraph;

pub struct Options {
    /// Maximum line width to wrap at.
    pub line_width: usize,
    /// How much a tab indent contributes to line width.
    pub tab_width: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            line_width: 80,
            tab_width: 4,
        }
    }
}

pub fn format(
    mut output: impl io::Write,
    paragaphs: &[Paragraph],
    options: Options,
) -> io::Result<()> {
    output.write(&[])?;
    Ok(())
}
