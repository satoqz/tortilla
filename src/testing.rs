use super::{Line, Token, Whitespace};

pub(super) trait LineExtension {
    fn trimmed(self) -> Self;
}

impl LineExtension for Line<'_> {
    fn trimmed(mut self) -> Self {
        self.newline = false;
        self
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

pub(super) fn whitespace_notation(pattern: &str) -> Whitespace<'static> {
    let token = match pattern.chars().next() {
        Some('s') => Token::Space,
        Some('t') => Token::Tab,
        _ => panic!("invalid whitespace pattern"),
    };

    let size = pattern[1..].parse().expect("invalid whitespace pattern");

    Whitespace(token, size)
}

#[macro_export]
macro_rules! line {
    ($indent:ident, $comment:expr, $padding:ident, $bullet:expr, $($word:expr),*) => {{
        use std::ops::Not;
        use crate::{Line, Token, testing::whitespace_notation};

        Line {
            indent: whitespace_notation(stringify!($indent)),
            comment: $comment.is_empty().not().then_some(Token::Word($comment)),
            padding: whitespace_notation(stringify!($padding)),
            bullet: $bullet.is_empty().not().then_some(Token::Word($bullet)),
            words: vec![$($word),*],
            newline: true,
        }
    }};

    ($indent:ident, $($word:expr),*) => {
        line!($indent, "", s0, "", $($word),*)
    };

    ($comment:expr, $padding:ident, $($word:expr),*) => {
        line!(s0, $comment, $padding, "", $($word),*)
    };

    ($($word:expr),*) => {
        line!(s0, "", s0, "", $($word),*)
    };
}
