mod lex;
mod merge;
mod parse;
mod wrap;

#[cfg(test)]
mod testing;

use lex::Lex;
use merge::Merge;
use parse::Parse;
use wrap::{Sauce, Wrap};

pub use wrap::Naive;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
    fn as_str(&self) -> &'static str {
        match self {
            Self::LF => "\n",
            Self::CRLF => "\r\n",
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Token<'t> {
    /// A space character (' ').
    Space,
    /// A tab character ('\t').
    Tab,
    /// A linefeed character ('\n') or a carriage return + linefeed character.
    Newline(Newline),
    /// A chain of characters without any whitespace.
    Word(&'t str),
}

impl<'t> Token<'t> {
    fn as_str(&self) -> &'t str {
        match self {
            Self::Space => " ",
            Self::Tab => "\t",
            Self::Word(s) => s,
            Self::Newline(newline) => newline.as_str(),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Whitespace<'t>(Token<'t>, usize);

#[derive(Debug, PartialEq)]
struct Line<'t> {
    indent: Whitespace<'t>,
    comment: Option<Token<'t>>,
    padding: Whitespace<'t>,
    bullet: Option<Token<'t>>,
    words: Vec<&'t str>,
    newline: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toppings {
    tabs: usize,
    width: usize,
    newline: Newline,
}

impl Default for Toppings {
    fn default() -> Self {
        Self {
            tabs: 4,
            width: 80,
            newline: Newline::default(),
        }
    }
}

impl Toppings {
    pub fn tabs(self, tabs: usize) -> Self {
        Self { tabs, ..self }
    }

    pub fn width(self, width: usize) -> Self {
        Self { width, ..self }
    }

    pub fn newline(self, newline: Newline) -> Self {
        Self { newline, ..self }
    }
}

pub fn wrap<S: Sauce>(input: &str, toppings: Toppings) -> impl Iterator<Item = &str> {
    let lines = Parse::new(Lex::new(input));
    Wrap::<Merge<Parse<Lex>>, S>::new(Merge::new(lines), toppings).map(|token| token.as_str())
}
