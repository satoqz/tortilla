use std::iter::Peekable;

use super::Token;

#[derive(Debug, PartialEq)]
pub struct Whitespace<'t> {
    pub token: Token<'t>,
    pub count: usize,
}

impl Whitespace<'static> {
    fn spaces(count: usize) -> Self {
        Self {
            token: Token::Space,
            count,
        }
    }

    fn tabs(count: usize) -> Self {
        Self {
            token: Token::Tab,
            count,
        }
    }

    fn zero() -> Self {
        Self::spaces(0)
    }
}

#[derive(Debug, PartialEq)]
pub struct Line<'t> {
    pub indent: Whitespace<'t>,
    pub comment: Option<Token<'t>>,
    pub padding: Whitespace<'t>,
    pub bullet: Option<Token<'t>>,
    pub words: Vec<&'t str>,
}

pub struct Parser<I: Iterator> {
    source: Peekable<I>,
}

pub fn parse<'t, I>(source: I) -> Parser<I>
where
    I: Iterator<Item = Token<'t>>,
{
    Parser {
        source: source.peekable(),
    }
}

impl<'t, I> Iterator for Parser<I>
where
    I: Iterator<Item = Token<'t>>,
{
    type Item = Line<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        self.source.peek()?;

        Some(Line {
            indent: self.whitespace(),
            comment: self.comment(),
            padding: self.whitespace(),
            bullet: self.bullet(),
            words: self.words(),
        })
    }
}

impl<'t, I> Parser<I>
where
    I: Iterator<Item = Token<'t>>,
{
    fn whitespace(&mut self) -> Whitespace<'t> {
        let Some(first) = self
            .source
            .next_if(|token| *token == Token::Space || *token == Token::Tab)
        else {
            return Whitespace {
                token: Token::Space,
                count: 0,
            };
        };

        let mut count = 1;
        while self.source.next_if_eq(&first).is_some() {
            count += 1;
        }

        Whitespace {
            token: first,
            count: count,
        }
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
                || (word.ends_with(&['.', ')'])
                    && word.len() > 1
                    && word
                        .chars()
                        .take(word.len() - 1)
                        .all(|c| c.is_ascii_digit()))
        })
    }

    fn words(&mut self) -> Vec<&'t str> {
        let mut words = Vec::new();

        while let Some(token) = self.source.next() {
            match token {
                Token::Space | Token::Tab => {}
                Token::Word(word) => words.push(word),
                Token::Newline => break,
            }
        }

        words
    }
}

#[cfg(test)]
mod tests {
    use super::Token::{self, *};
    use super::{Line, Whitespace};

    fn parse<'t>(tokens: &[Token<'t>]) -> Vec<Line<'t>> {
        super::parse(tokens.iter().copied()).collect()
    }

    #[test]
    fn idk() {
        assert_eq!(
            parse(&[
                Tab,
                Tab,
                Word("//"),
                Space,
                Word("1."),
                Space,
                Word("hi"),
                Word("hello")
            ]),
            vec![Line {
                indent: Whitespace::tabs(2),
                comment: Some(Token::Word("//")),
                padding: Whitespace::spaces(1),
                bullet: Some(Token::Word("1.")),
                words: vec!["hi", "hello"],
            }]
        );
    }
}
