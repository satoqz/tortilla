use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};

use super::{Newline, Token};

enum State {
    Clean,
    Word(usize),
}

pub(super) struct Lex<'t> {
    input: &'t str,
    inner: GraphemeIndices<'t>,
    state: State,
    pending: Option<Token<'static>>,
}

impl<'t> Lex<'t> {
    pub fn new(input: &'t str) -> Self {
        Self {
            input,
            state: State::Clean,
            inner: input.grapheme_indices(true),
            pending: None,
        }
    }
}

fn word_break(grapheme: &str) -> Option<Token<'static>> {
    Some(match grapheme {
        " " => Token::Space,
        "\t" => Token::Tab,
        "\n" => Token::Newline(Newline::LF),
        "\r\n" => Token::Newline(Newline::CRLF),
        _ => return None,
    })
}

impl<'t> Iterator for Lex<'t> {
    type Item = Token<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token) = self.pending.take() {
            return Some(token);
        }

        for (byte_idx, grapheme) in self.inner.by_ref() {
            match self.state {
                State::Clean => {
                    if let Some(token) = word_break(grapheme) {
                        return Some(token);
                    } else {
                        self.state = State::Word(byte_idx);
                    }
                }

                State::Word(start_idx) => {
                    if let Some(token) = word_break(grapheme) {
                        self.state = State::Clean;
                        self.pending = Some(token);
                        return Some(Token::Word(&self.input[start_idx..byte_idx]));
                    }
                }
            }
        }

        if let State::Word(start_idx) = self.state {
            self.state = State::Clean;
            return Some(Token::Word(&self.input[start_idx..]));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{Token, tokens};

    fn lex(input: &str) -> Vec<Token<'_>> {
        super::Lex::new(input).collect()
    }
}
