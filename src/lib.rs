mod lex;
mod transform;

pub use lex::*;
pub use transform::*;

use std::fmt;
use std::io;

#[derive(Debug, Clone)]
pub struct Options {
    /// Maximum line width to wrap at.
    pub line_width: usize,
    /// How much a tab indent contributes to line width.
    pub tab_width: usize,
    pub newline: Newline,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            line_width: 80,
            tab_width: 4,
            newline: Newline::default(),
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
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Token<'a> {
    /// A space character (' ').
    Space,
    /// A tab character ('\t').
    Tab,
    /// A linefeed character ('\n') or a carriage return + linefeed character.
    Newline,
    /// A chain of characters without any whitespace.
    Word(&'a str),
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
pub fn write<'a, I>(mut output: impl io::Write, tokens: I, newline: Newline) -> io::Result<()>
where
    I: Iterator<Item = Token<'a>>,
{
    for token in tokens {
        output.write(token.as_bytes(newline))?;
    }

    Ok(())
}

/// Convenience function to write a token stream out to a [std::fmt::Write].
pub fn format<'a, I>(mut output: impl fmt::Write, tokens: I, newline: Newline) -> fmt::Result
where
    I: Iterator<Item = Token<'a>>,
{
    for token in tokens {
        output.write_str(token.as_str(newline))?;
    }

    Ok(())
}

/// End-to-end wrapping of a string.
pub fn wrap(input: &str, options: &Options) -> String {
    transform(lex(&input), options)
        .map(|token| token.as_str(options.newline).to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todo() {
        assert_eq!(&wrap("foo bar", &Options::default()), "foo bar")
    }
}
