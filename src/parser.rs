use super::lexer::Token;

pub struct Paragraph<'a> {
    /// The actual contents of the paragraph, without whitespace.
    pub words: Vec<&'a str>,
    /// The paragraph's leading whitespace, i.e., indent, if any.
    pub indent: Option<Indent>,
    /// The paragraph's comment token (following) the indent, if any.
    pub comment: Option<&'a str>,
    /// The paragraph's bullet point / list token, if any.
    pub bullet: Option<&'a str>,
    /// The newline token for this paragraph (LF or CRLF).
    pub newline: &'a str,
    /// If this paragraph should print with a final newline.
    pub final_newline: bool,
}

pub enum Whitespace {
    /// ' '
    Space,
    /// '\t'
    Tab,
}

pub struct Indent {
    /// The whitespace token for this indent.
    pub whitespace: Whitespace,
    /// Size of the indent, i.e., how many times whitespace repeats.
    pub size: usize,
}

#[derive(Debug)]
pub enum Error {}

pub fn parse<'a>(tokens: &[Token<'a>]) -> Result<Vec<Paragraph<'a>>, Error> {
    Ok(Vec::new())
}
