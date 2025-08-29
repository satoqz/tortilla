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

pub use wrap::{Guacamole, Salsa};

/// Newline characters.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Newline {
    /// A line feed (`\n`).
    LF,
    /// A carriage return + line feed (`\r\n`).
    CRLF,
}

impl Default for Newline {
    /// The default newline character (`\n`).
    fn default() -> Self {
        Self::LF
    }
}

impl Newline {
    /// String representation of the newline character.
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
    /// A tab character (`\t`).
    Tab,
    /// A newline character, see [Newline].
    Newline(Newline),
    /// One or more graphemes devoid of any of the above characters.
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

/// Parameters for line breaking algorithms & formatting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toppings {
    tabs: usize,
    width: usize,
    newline: Newline,
}

impl Default for Toppings {
    /// Default configuration with maximum width 80, tab width 4 and LF (`\n`)
    /// newlines.
    fn default() -> Self {
        Self {
            tabs: 4,
            width: 80,
            newline: Newline::default(),
        }
    }
}

impl Toppings {
    /// The targeted maximum line width (Unicode width). The default value is
    /// 80, some other common choices are 72, 100 and 120.
    ///
    /// Note that tortilla does not respect this setting in the following cases:
    ///
    /// 1. Any combination of indentation, comment token, bullet token and/or
    ///    indentation of the bullet token following a comment token is never
    ///    wrapped, and may exceed maximum line width by itself.
    ///
    /// 2. Words that exceed maximum line width by themselves (or in combination
    ///    with case 1.) are not broken apart and get placed on their own line.
    pub fn width(self, width: usize) -> Self {
        Self { width, ..self }
    }

    /// How much a tab character (`\t`) contributes to line width calculation.
    /// The default value is 4.
    ///
    /// While a tab character has a width of 1 as per the Unicode standard
    /// (equal to the width of a single space), code editors commonly display
    /// tabs wider than a space character, typically as 2, 4 or 8 spaces.
    /// Special-casing tab width thus helps make output look more natural,
    /// especially in comments that are preceded by several levels of
    /// indentation.
    pub fn tabs(self, tabs: usize) -> Self {
        Self { tabs, ..self }
    }

    /// The newline character to use, see [Newline]. This is a line feed
    /// character (`\n`, [Newline::LF]) by default.
    ///
    /// tortilla does not perform any heuristical newline character detection
    /// and always outputs uniform linebreaks. You may choose to perform such
    /// detection on the input string beforehand, and then pass the appropriate
    /// variant to tortilla.
    pub fn newline(self, newline: Newline) -> Self {
        Self { newline, ..self }
    }
}

/// Wrap text. Output is lazily generated and returned in small chunks.
///
/// To set the line breaking algorithm, see [Guacamole] and [Salsa]. For other
/// options, see [Toppings].
///
/// # Examples
///
/// Wrap a string and collect it into a new string:
///
/// ```
/// use tortilla::{wrap, Salsa, Toppings};
///
/// let input = "
/// - foo bar baz
/// ";
///
/// let toppings = Toppings::default().width(8);
/// let output = wrap::<Salsa>(input, toppings).collect::<String>();
///
/// assert_eq!(output, "
/// - foo
///   bar
///   baz
/// ");
/// ```
///
/// Stream output to stdout:
///
/// ```
/// use std::io::{self, Write};
/// use tortilla::{wrap, Salsa, Toppings};
///
/// let input = "...";
///
/// let toppings = Toppings::default();
/// let mut stdout = io::stdout().lock();
///
/// for chunk in wrap::<Salsa>(input, toppings) {
///     stdout.write_all(chunk.as_bytes()).unwrap();
/// }
/// ```
///
pub fn wrap<S: Sauce>(input: &str, toppings: Toppings) -> impl Iterator<Item = &str> {
    let lines = Parse::new(Lex::new(input));
    Wrap::<Merge<Parse<Lex>>, S>::new(Merge::new(lines), toppings).map(|token| token.as_str())
}
