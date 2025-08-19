use std::iter::Peekable;

use super::{Line, Token, Whitespace};

pub(super) struct Parse<I: Iterator> {
    source: Peekable<I>,
}

pub(super) fn iter<'t, I>(source: I) -> Parse<I>
where
    I: Iterator<Item = Token<'t>>,
{
    Parse {
        source: source.peekable(),
    }
}

impl<'t, I> Iterator for Parse<I>
where
    I: Iterator<Item = Token<'t>>,
{
    type Item = Line<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        self.source.peek()?;

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
    fn whitespace(&mut self) -> Whitespace<'t> {
        let Some(first) = self
            .source
            .next_if(|token| *token == Token::Space || *token == Token::Tab)
        else {
            return Whitespace(Token::Space, 0);
        };

        let mut count = 1;
        while self.source.next_if_eq(&first).is_some() {
            count += 1;
        }

        Whitespace(first, count)
    }

    fn comment(&mut self) -> Option<Token<'t>> {
        const COMMENT_TOKENS: &[&str] = &["#", ">", ";", "//", "--", ";;", "///", "//!"];

        self.source.next_if(|token| match token {
            Token::Word(word) => COMMENT_TOKENS.contains(word),
            _ => false,
        })
    }

    fn bullet(&mut self) -> Option<Token<'t>> {
        self.source.next_if(|token| {
            let Token::Word(word) = token else {
                return false;
            };

            ["-", "*", "•"].contains(word)
                || (word.ends_with(['.', ')'])
                    && word.len() > 1
                    && word
                        .chars()
                        .take(word.len() - 1)
                        .all(|c| c.is_ascii_digit()))
        })
    }

    fn words(&mut self) -> (Vec<&'t str>, bool) {
        let mut words = Vec::new();

        for token in self.source.by_ref() {
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
    use crate::{Line, Token, line, testing::LineExtension, tokens};

    fn parse<'t>(tokens: Vec<Token<'t>>) -> Vec<Line<'t>> {
        super::iter(tokens.into_iter()).collect()
    }

    #[test]
    fn idk() {
        assert_eq!(
            parse(tokens!("//", s, "1.", s, "hi", "hello")),
            vec![line!(s0, "//", s1, "1.", "hi", "hello").trimmed()],
        );

        assert_eq!(
            parse(tokens!(t, t, "//", s, "1.", s, "hi", "hello")),
            vec![line!(t2, "//", s1, "1.", "hi", "hello").trimmed()]
        );

        assert_eq!(
            parse(tokens!(lf, "testing", lf, lf)),
            vec![
                line!(s0, "", s0, ""),
                line!(s0, "", s0, "", "testing"),
                line!(s0, "", s0, "")
            ]
        )
    }
}
