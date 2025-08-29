use super::{Line, Token, Whitespace};

pub trait LineExtension {
    fn trimmed(self) -> Self;
}

impl LineExtension for Line<'_> {
    fn trimmed(self) -> Self {
        Self {
            newline: false,
            ..self
        }
    }
}

#[macro_export]
macro_rules! token {
    (s) => {
        crate::Token::Space
    };
    (t) => {
        crate::Token::Tab
    };
    (lf) => {
        crate::Token::Newline(crate::Newline::LF)
    };
    (crlf) => {
        crate::Token::Newline(crate::Newline::CRLF)
    };
    ($word:expr) => {
        crate::Token::Word($word)
    };
}

#[macro_export]
macro_rules! tokens {
    ($($token:tt),*) => {
        vec![$(crate::token!($token)),*]
    };

}

pub fn whitespace_notation(pattern: &str) -> Whitespace {
    let count = pattern[1..].parse().expect("invalid whitespace pattern");

    match pattern.chars().next() {
        Some('s') => Whitespace::Space(count),
        Some('t') => Whitespace::Tab(count),
        _ => panic!("invalid whitespace pattern"),
    }
}

#[macro_export]
macro_rules! line {
    ($indent:ident, $comment:expr, $padding:ident, $bullet:expr $(, $($word:expr),*)?) => {{
        use std::ops::Not;
        use crate::{Line, Token, testing::whitespace_notation};

        Line {
            indent: whitespace_notation(stringify!($indent)),
            comment: $comment.is_empty().not().then_some(Token::Word($comment)),
            padding: whitespace_notation(stringify!($padding)),
            bullet: $bullet.is_empty().not().then_some(Token::Word($bullet)),
            words: vec![$($($word),*)?],
            newline: true,
        }
    }};
}
