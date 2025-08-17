mod lex;
mod transform;

pub use lex::*;
pub use transform::*;

use std::fmt;
use std::io;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Token<'a> {
    /// ' '
    Space,
    /// '\t'
    Tab,
    /// '\n'
    NewlineLF,
    /// '\r\n'
    NewlineCRLF,
    /// A chain of characters without any whitespace.
    Word(&'a str),
}

impl Token<'_> {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Space => " ",
            Self::Tab => "\t",
            Self::NewlineLF => "\n",
            Self::NewlineCRLF => "\r\n",
            Self::Word(s) => s,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

pub fn write<'a, I>(mut output: impl io::Write, tokens: I) -> io::Result<()>
where
    I: Iterator<Item = Token<'a>>,
{
    for token in tokens {
        output.write(token.as_bytes())?;
    }

    Ok(())
}

pub fn format<'a, I>(mut output: impl fmt::Write, tokens: I) -> fmt::Result
where
    I: Iterator<Item = Token<'a>>,
{
    for token in tokens {
        output.write_str(token.as_str())?;
    }

    Ok(())
}
