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

    /// A pending word break token that hasn't returned yet.
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

    #[test]
    fn test_empty_input() {
        assert_eq!(lex(""), tokens!());
    }

    #[test]
    fn test_single_space() {
        assert_eq!(lex(" "), tokens!(s));
    }

    #[test]
    fn test_single_tab() {
        assert_eq!(lex("\t"), tokens!(t));
    }

    #[test]
    fn test_single_lf() {
        assert_eq!(lex("\n"), tokens!(lf));
    }

    #[test]
    fn test_single_crlf() {
        assert_eq!(lex("\r\n"), tokens!(crlf));
    }

    #[test]
    fn test_single_word() {
        assert_eq!(lex("hello"), tokens!("hello"));
    }

    #[test]
    fn test_mixed_tokens() {
        assert_eq!(
            lex("hello \tworld\n\r\nnext\tline"),
            tokens!("hello", s, t, "world", lf, crlf, "next", t, "line")
        );
    }

    #[test]
    fn test_multiple_spaces() {
        assert_eq!(lex("   "), tokens!(s, s, s));
    }

    #[test]
    fn test_multiple_tabs() {
        assert_eq!(lex("\t\t\t"), tokens!(t, t, t));
    }

    #[test]
    fn test_multiple_newlines() {
        assert_eq!(lex("\n\n\r\n"), tokens!(lf, lf, crlf));
    }

    #[test]
    fn test_word_with_spaces_and_tabs() {
        assert_eq!(lex("a b\tc\nd"), tokens!("a", s, "b", t, "c", lf, "d"));
    }

    #[test]
    fn test_word_with_mixed_newlines() {
        assert_eq!(lex("a\nb\r\nc"), tokens!("a", lf, "b", crlf, "c"));
    }

    #[test]
    fn test_only_newlines() {
        assert_eq!(lex("\n\r\n\n"), tokens!(lf, crlf, lf));
    }

    #[test]
    fn test_lone_cr_in_word() {
        assert_eq!(lex("a\rb"), tokens!("a\rb"));
    }

    #[test]
    fn test_cr_not_followed_by_lf() {
        assert_eq!(lex("a\r"), tokens!("a\r"));
    }

    #[test]
    fn test_crlf_as_newline() {
        assert_eq!(lex("a\r\nb"), tokens!("a", crlf, "b"));
    }

    #[test]
    fn test_emoji() {
        assert_eq!(lex("hello🇩🇪world"), tokens!("hello🇩🇪world"));
    }

    #[test]
    fn test_combining_mark() {
        assert_eq!(lex("café"), tokens!("café"));
    }

    #[test]
    fn test_multibyte_chars() {
        assert_eq!(lex("汉字 test"), tokens!("汉字", s, "test"));
    }

    #[test]
    fn test_emoji_with_spaces() {
        assert_eq!(lex("hello 🇩🇪 world"), tokens!("hello", s, "🇩🇪", s, "world"));
    }

    #[test]
    fn test_complex_grapheme_clusters() {
        assert_eq!(
            lex("Zöe\tétoile\n\r\n👨‍👩‍👧‍👦"),
            tokens!("Zöe", t, "étoile", lf, crlf, "👨‍👩‍👧‍👦")
        );
    }

    #[test]
    fn test_nbsp_as_part_of_word() {
        assert_eq!(lex("hello\u{A0}world"), tokens!("hello\u{A0}world"));
    }

    #[test]
    fn test_mixed_unicode_and_ascii() {
        assert_eq!(
            lex("hello\u{A0}🌍\tworld\n\r\nnext\u{2009}line"),
            tokens!("hello\u{A0}🌍", t, "world", lf, crlf, "next\u{2009}line")
        );
    }
}
