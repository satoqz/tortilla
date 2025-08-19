mod lex;
mod merge;
mod parse;
mod wrap;

#[cfg(test)]
mod testing;

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
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LF => "\n",
            Self::CRLF => "\r\n",
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token<'t> {
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
    pub fn as_str(&self) -> &'t str {
        match self {
            Self::Space => " ",
            Self::Tab => "\t",
            Self::Word(s) => s,
            Self::Newline(newline) => newline.as_str(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

#[derive(Debug, PartialEq)]
struct Whitespace<'t>(Token<'t>, usize);

#[cfg(test)]
impl Whitespace<'static> {
    fn zero() -> Self {
        Self(Token::Space, 0)
    }

    fn spaces(count: usize) -> Self {
        Self(Token::Space, count)
    }

    fn tabs(count: usize) -> Self {
        Self(Token::Tab, count)
    }
}

#[derive(Debug, PartialEq)]
struct Line<'t> {
    indent: Whitespace<'t>,
    comment: Option<Token<'t>>,
    padding: Whitespace<'t>,
    bullet: Option<Token<'t>>,
    words: Vec<&'t str>,
    newline: bool,
}

#[derive(Debug, Clone)]
pub struct Toppings {
    /// Maximum line width to wrap at.
    pub line_width: usize,
    /// How much a tab indent contributes to line width.
    pub tab_size: usize,

    pub newline: Newline,
}

impl Default for Toppings {
    fn default() -> Self {
        Self {
            line_width: 80,
            tab_size: 4,
            newline: Newline::default(),
        }
    }
}

impl Toppings {
    pub fn line_width(self, n: usize) -> Self {
        Self {
            line_width: n,
            ..self
        }
    }

    pub fn tab_size(self, n: usize) -> Self {
        Self {
            tab_size: n,
            ..self
        }
    }
}

pub fn wrap<'t>(input: &'t str, toppings: Toppings) -> impl Iterator<Item = Token<'t>> {
    let lines = parse::iter(lex::iter(input));
    wrap::iter(merge::iter(lines), toppings)
}
