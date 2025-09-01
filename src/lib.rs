mod lex;
mod merge;
mod parse;
mod wrap;

use lex::Lex;
use merge::Merge;
use parse::Parse;
use wrap::{LineWrap, Sauce};

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

#[derive(Debug, Clone, Copy, PartialEq)]
enum Whitespace {
    /// A space character (' ') repeated n times.
    Space(usize),
    /// A tab character (`\t`) repeated n times.
    Tab(usize),
}

impl Whitespace {
    /// Repetition count of the whitespace.
    fn count(&self) -> usize {
        match self {
            Self::Space(c) | Self::Tab(c) => *c,
        }
    }

    /// String representation of the whitespace character, repeated only once.
    fn as_str(&self) -> &'static str {
        match self {
            Self::Space(_) => " ",
            Self::Tab(_) => "\t",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Line<'t> {
    indent: Whitespace,
    comment: Option<&'t str>,
    padding: Whitespace,
    bullet: Option<&'t str>,
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
/// // Leading indent of two spaces, comment token and bullet point:
/// let input = "
///   // - foo bar baz
/// ";
///
/// // Set a low width that will force wrapping across several lines:
/// let toppings = Toppings::default().width(8);
/// let output = wrap::<Salsa>(input, toppings).collect::<String>();
///
/// // Indent and comment token are replicated onto new lines, bullet point
/// // remains only on the first line and is replaced by spacing on the
/// // subsequent ones:
/// assert_eq!(output, "
///   // - foo
///   //   bar
///   //   baz
/// ");
///
/// // Now set a much higher width, which will get us back to the original input
/// // (a single line):
/// let toppings = Toppings::default().width(100);
/// let output = wrap::<Salsa>(&output, toppings).collect::<String>();
///
/// assert_eq!(output, input);
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
pub fn wrap<S: Sauce>(input: &str, toppings: Toppings) -> Wrap<'_, S> {
    Wrap {
        toppings,
        lines: Merge::new(Parse::new(Lex::new(input))),
        current: None,
    }
}

/// An [Iterator] over chunks of wrapped output.
pub struct Wrap<'t, S> {
    toppings: Toppings,
    lines: Merge<Parse<Lex<'t>>>,
    current: Option<LineWrap<'t, S>>,
}

impl<'t, S: Sauce> Iterator for Wrap<'t, S> {
    type Item = &'t str;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let inner = match &mut self.current {
                Some(inner) => inner,
                None => self
                    .current
                    .insert(LineWrap::new(self.lines.next()?, &self.toppings)),
            };

            match inner.next() {
                Some(chunk) => return Some(chunk),
                None => self.current = None,
            }
        }
    }
}

/// Utility macro to construct a [Token].
#[cfg(test)]
#[macro_export]
macro_rules! token {
    { s } => { $crate::Token::Space };
    { t } => { $crate::Token::Tab };
    { lf } => { $crate::Token::Newline($crate::Newline::LF) };
    { crlf } => { $crate::Token::Newline($crate::Newline::CRLF) };
    { $word:expr } => { $crate::Token::Word($word) };
}

/// Utility macro to construct a [Vec]<[Token]>.
#[cfg(test)]
#[macro_export]
macro_rules! tokens {
    [$($token:tt),*] => { vec![$($crate::token!{ $token }),*] };
}

/// Utility macro to construct a [Line].
#[cfg(test)]
#[macro_export]
macro_rules! line {
    (
        $indent:expr, $comment:expr,
        $padding:expr, $bullet:expr
        $(, $($word:expr),*)?
    ) => {
        $crate::Line {
            indent: $indent, comment: $comment,
            padding: $padding, bullet: $bullet,
            words: vec![$($($word),*)?], newline: false,
        }
    };

    (
        $indent:expr, $comment:expr,
        $padding:expr, $bullet:expr
        $(, $($word:expr),*)? ;
    ) => {
        $crate::Line {
            indent: $indent, comment: $comment,
            padding: $padding, bullet: $bullet,
            words: vec![$($($word),*)?], newline: true,
        }
    };
}
