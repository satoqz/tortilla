mod lex;
mod transform;

pub use lex::*;
pub use transform::*;

use std::fmt;
use std::io;

use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
pub struct Options {
    /// Maximum line width to wrap at.
    pub line_width: usize,
    /// How much a tab indent contributes to line width.
    pub tab_width: usize,
    pub newline: Option<Newline>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            line_width: 80,
            tab_width: 4,
            newline: None,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Newline {
    LF,
    CRLF,
}

impl Default for Newline {
    fn default() -> Self {
        Self::LF
    }
}

impl Newline {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LF => "\n",
            Self::CRLF => "\r\n",
        }
    }

    pub fn first_in(input: &str) -> Option<Newline> {
        input.graphemes(true).find_map(|grapheme| match grapheme {
            "\n" => Some(Newline::LF),
            "\r\n" => Some(Newline::CRLF),
            _ => None,
        })
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Token<'t> {
    /// A space character (' ').
    Space,
    /// A tab character ('\t').
    Tab,
    /// A linefeed character ('\n') or a carriage return + linefeed character.
    Newline,
    /// A chain of characters without any whitespace.
    Word(&'t str),
}

impl Token<'_> {
    pub fn as_str(&self, newline: Newline) -> &str {
        match self {
            Self::Space => " ",
            Self::Tab => "\t",
            Self::Word(s) => s,
            Self::Newline => newline.as_str(),
        }
    }

    pub fn as_bytes(&self, newline: Newline) -> &[u8] {
        self.as_str(newline).as_bytes()
    }
}

/// Convenience function to write a token stream out to a [std::io::Write].
pub fn write_all<'t, I>(mut output: impl io::Write, tokens: I, newline: Newline) -> io::Result<()>
where
    I: Iterator<Item = Token<'t>>,
{
    for token in tokens {
        output.write_all(token.as_bytes(newline))?;
    }

    Ok(())
}

/// Convenience function to write a token stream out to a [std::fmt::Write].
pub fn format<'t, I>(mut output: impl fmt::Write, tokens: I, newline: Newline) -> fmt::Result
where
    I: Iterator<Item = Token<'t>>,
{
    for token in tokens {
        output.write_str(token.as_str(newline))?;
    }

    Ok(())
}

/// End-to-end wrapping of a string.
pub fn wrap(input: &str, options: Options) -> String {
    let newline = options
        .newline
        .unwrap_or(Newline::first_in(input).unwrap_or_default());

    transform(lex(&input), options)
        .map(|token| token.as_str(newline).to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todo() {
        assert_eq!(&wrap("foo bar", Options::default()), "foo bar")
    }
}
