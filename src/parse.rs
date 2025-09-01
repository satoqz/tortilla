use std::iter::Peekable;

use super::{Line, Token, Whitespace};

pub(super) struct Parse<I: Iterator> {
    tokens: Peekable<I>,
}

impl<I: Iterator> Parse<I> {
    pub fn new(tokens: I) -> Self {
        Self {
            tokens: tokens.peekable(),
        }
    }

    fn lookahead<F, T>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&I::Item) -> Option<T>,
    {
        let mut ret = None;

        self.tokens.next_if(|token| {
            ret = f(token);
            ret.is_some()
        });

        ret
    }
}

impl<'t, I> Iterator for Parse<I>
where
    I: Iterator<Item = Token<'t>>,
{
    type Item = Line<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.peek()?;

        let indent = self.whitespace();
        let comment = self.comment();
        let padding = self.whitespace();
        let bullet = self.bullet();
        let (words, newline) = self.words();

        Some(Line {
            indent,
            comment,
            padding,
            bullet,
            words,
            newline,
        })
    }
}

impl<'t, I> Parse<I>
where
    I: Iterator<Item = Token<'t>>,
{
    fn whitespace(&mut self) -> Whitespace {
        let Some(first) = self
            .tokens
            .next_if(|token| *token == Token::Space || *token == Token::Tab)
        else {
            return Whitespace::Space(0);
        };

        let mut count = 1;
        while self.tokens.next_if_eq(&first).is_some() {
            count += 1;
        }

        match first {
            Token::Space => Whitespace::Space(count),
            Token::Tab => Whitespace::Tab(count),
            _ => unreachable!(),
        }
    }

    fn comment(&mut self) -> Option<&'t str> {
        const COMMENT_TOKENS: &[&str] = &["#", ">", ";", "//", "--", ";;", "///", "//!"];

        self.lookahead(|token| match token {
            Token::Word(word) => COMMENT_TOKENS.contains(word).then_some(*word),
            _ => None,
        })
    }

    fn bullet(&mut self) -> Option<&'t str> {
        self.lookahead(|token| {
            let Token::Word(word) = token else {
                return None;
            };

            let is_bullet = ["-", "*", "•"].contains(word)
                || (word.ends_with(['.', ')'])
                    && word.len() > 1
                    && word
                        .chars()
                        .take(word.len() - 1)
                        .all(|c| c.is_ascii_digit()));

            is_bullet.then_some(*word)
        })
    }

    fn words(&mut self) -> (Vec<&'t str>, bool) {
        let mut words = Vec::new();

        for token in self.tokens.by_ref() {
            match token {
                Token::Space | Token::Tab => {}
                Token::Word(word) => words.push(word),
                Token::Newline(_) => return (words, true),
            }
        }

        (words, false)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Line, Token, Whitespace::*};
    use crate::{line, tokens};

    fn parse(tokens: Vec<Token>) -> Vec<Line> {
        super::Parse::new(tokens.into_iter()).collect()
    }

    #[test]
    fn empty() {
        assert_eq!(parse(tokens![]), vec![]);
    }

    #[test]
    fn single_space_indent() {
        assert_eq!(
            parse(tokens![s]),
            vec![line!(Space(1), None, Space(0), None)]
        );
    }

    #[test]
    fn single_tab_indent() {
        assert_eq!(parse(tokens![t]), vec![line!(Tab(1), None, Space(0), None)]);
    }

    #[test]
    fn multiple_spaces_indent() {
        assert_eq!(
            parse(tokens![s, s, s, s]),
            vec![line!(Space(4), None, Space(0), None)]
        );
    }

    #[test]
    fn multiple_tabs_indent() {
        assert_eq!(
            parse(tokens![t, t]),
            vec![line!(Tab(2), None, Space(0), None)]
        );
    }

    #[test]
    fn spaces_then_tabs() {
        assert_eq!(
            parse(tokens![s, s, s, t, t]),
            vec![line!(Space(3), None, Tab(2), None)]
        );
    }

    #[test]
    fn tabs_then_spaces() {
        assert_eq!(
            parse(tokens![t, t, t, s, s]),
            vec![line!(Tab(3), None, Space(2), None)]
        );
    }

    #[test]
    fn comment_only() {
        assert_eq!(
            parse(tokens!["#"]),
            vec![line!(Space(0), Some("#"), Space(0), None)]
        );
        assert_eq!(
            parse(tokens!["//"]),
            vec![line!(Space(0), Some("//"), Space(0), None)]
        );
    }

    #[test]
    fn indented_comment() {
        assert_eq!(
            parse(tokens![s, s, s, s, "#"]),
            vec![line!(Space(4), Some("#"), Space(0), None)]
        );
        assert_eq!(
            parse(tokens![t, "//"]),
            vec![line!(Tab(1), Some("//"), Space(0), None)]
        );
    }

    #[test]
    fn indented_comment_and_padding() {
        assert_eq!(
            parse(tokens![s, s, s, s, "#", t, s]),
            vec![line!(Space(4), Some("#"), Tab(1), None)]
        );
    }

    #[test]
    fn bullets() {
        assert_eq!(
            parse(tokens!["-"]),
            vec![line!(Space(0), None, Space(0), Some("-"))]
        );
        assert_eq!(
            parse(tokens!["123."]),
            vec![line!(Space(0), None, Space(0), Some("123."))]
        );
    }

    #[test]
    fn indented_bullets() {
        assert_eq!(
            parse(tokens![s, s, s, s, "-"]),
            vec![line!(Space(4), None, Space(0), Some("-"))]
        );
        assert_eq!(
            parse(tokens![t, "123."]),
            vec![line!(Tab(1), None, Space(0), Some("123."))]
        );
    }

    #[test]
    fn comment_and_bullet() {
        assert_eq!(
            parse(tokens![t, "//", s, "-"]),
            vec![line!(Tab(1), Some("//"), Space(1), Some("-"))]
        );
    }

    #[test]
    fn words() {
        assert_eq!(
            parse(tokens!["foo", s, s, "bar", t, "baz"]),
            vec![line!(Space(0), None, Space(0), None, "foo", "bar", "baz")]
        );
    }

    #[test]
    fn all_together() {
        assert_eq!(
            parse(tokens![
                t, t, "//", s, s, s, "-", s, s, "foo", s, s, "bar", t, "baz"
            ]),
            vec![line!(
                Tab(2),
                Some("//"),
                Space(3),
                Some("-"),
                "foo",
                "bar",
                "baz"
            )]
        );
    }

    #[test]
    fn newlines() {
        assert_eq!(
            parse(tokens!["foo", "bar", lf, crlf, "baz"]),
            vec![
                line!(Space(0), None, Space(0), None, "foo", "bar" ;),
                line!(Space(0), None, Space(0), None ;),
                line!(Space(0), None, Space(0), None, "baz"),
            ]
        );
    }
}
