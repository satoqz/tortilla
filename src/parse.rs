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
    use crate::{Line, Token, line, testing::LineExtension, tokens};

    fn parse(tokens: Vec<Token>) -> Vec<Line> {
        super::Parse::new(tokens.into_iter()).collect()
    }
}
