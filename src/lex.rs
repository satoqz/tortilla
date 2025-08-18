use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};

use super::Token;

enum State {
    Clean,
    Word(usize),
}

pub struct Lexer<'t> {
    input: &'t str,
    inner: GraphemeIndices<'t>,

    state: State,

    /// A pending word break token that hasn't returned yet.
    pending: Option<Token<'static>>,
}

pub fn lex(input: &str) -> Lexer<'_> {
    Lexer {
        input,
        state: State::Clean,
        inner: input.grapheme_indices(true),
        pending: None,
    }
}

fn word_break(grapheme: &str) -> Option<Token<'static>> {
    Some(match grapheme {
        " " => Token::Space,
        "\t" => Token::Tab,
        "\n" | "\r\n" => Token::Newline,
        _ => return None,
    })
}

impl<'t> Iterator for Lexer<'t> {
    type Item = Token<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token) = self.pending.take() {
            return Some(token);
        }

        while let Some((byte_idx, grapheme)) = self.inner.next() {
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
    use super::Token::{self, *};

    fn lex(input: &str) -> Vec<Token<'_>> {
        super::lex(input).collect()
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(lex(""), vec![]);
    }

    #[test]
    fn test_single_space() {
        assert_eq!(lex(" "), vec![Space]);
    }

    #[test]
    fn test_single_tab() {
        assert_eq!(lex("\t"), vec![Tab]);
    }

    #[test]
    fn test_single_lf() {
        assert_eq!(lex("\n"), vec![Newline]);
    }

    #[test]
    fn test_single_crlf() {
        assert_eq!(lex("\r\n"), vec![Newline]);
    }

    #[test]
    fn test_single_word() {
        assert_eq!(lex("hello"), vec![Word("hello")]);
    }

    #[test]
    fn test_mixed_tokens() {
        assert_eq!(
            lex("hello \tworld\n\r\nnext\tline"),
            vec![
                Word("hello"),
                Space,
                Tab,
                Word("world"),
                Newline,
                Newline,
                Word("next"),
                Tab,
                Word("line"),
            ]
        );
    }

    #[test]
    fn test_multiple_spaces() {
        assert_eq!(lex("   "), vec![Space, Space, Space]);
    }

    #[test]
    fn test_multiple_tabs() {
        assert_eq!(lex("\t\t\t"), vec![Tab, Tab, Tab]);
    }

    #[test]
    fn test_multiple_newlines() {
        assert_eq!(lex("\n\n\r\n"), vec![Newline, Newline, Newline]);
    }

    #[test]
    fn test_word_with_spaces_and_tabs() {
        assert_eq!(
            lex("a b\tc\nd"),
            vec![
                Word("a"),
                Space,
                Word("b"),
                Tab,
                Word("c"),
                Newline,
                Word("d"),
            ]
        );
    }

    #[test]
    fn test_word_with_mixed_newlines() {
        assert_eq!(
            lex("a\nb\r\nc"),
            vec![Word("a"), Newline, Word("b"), Newline, Word("c"),]
        );
    }

    #[test]
    fn test_only_newlines() {
        assert_eq!(lex("\n\r\n\n"), vec![Newline, Newline, Newline,]);
    }

    #[test]
    fn test_lone_cr_in_word() {
        assert_eq!(lex("a\rb"), vec![Word("a\rb")]);
    }

    #[test]
    fn test_cr_not_followed_by_lf() {
        assert_eq!(lex("a\r"), vec![Word("a\r")]);
    }

    #[test]
    fn test_crlf_as_newline() {
        assert_eq!(lex("a\r\nb"), vec![Word("a"), Newline, Word("b")]);
    }

    #[test]
    fn test_emoji() {
        assert_eq!(lex("hello🇩🇪world"), vec![Word("hello🇩🇪world")]);
    }

    #[test]
    fn test_combining_mark() {
        assert_eq!(lex("café"), vec![Word("café")]);
    }

    #[test]
    fn test_multibyte_chars() {
        assert_eq!(lex("汉字 test"), vec![Word("汉字"), Space, Word("test")]);
    }

    #[test]
    fn test_emoji_with_spaces() {
        assert_eq!(
            lex("hello 🇩🇪 world"),
            vec![Word("hello"), Space, Word("🇩🇪"), Space, Word("world"),]
        );
    }

    #[test]
    fn test_complex_grapheme_clusters() {
        assert_eq!(
            lex("Zöe\tétoile\n\r\n👨‍👩‍👧‍👦"),
            vec![
                Word("Zöe"),
                Tab,
                Word("étoile"),
                Newline,
                Newline,
                Word("👨‍👩‍👧‍👦"),
            ]
        );
    }

    #[test]
    fn test_nbsp_as_part_of_word() {
        assert_eq!(lex("hello\u{A0}world"), vec![Word("hello\u{A0}world")]);
    }

    #[test]
    fn test_mixed_unicode_and_ascii() {
        assert_eq!(
            lex("hello\u{A0}🌍\tworld\n\r\nnext\u{2009}line"),
            vec![
                Word("hello\u{A0}🌍"),
                Tab,
                Word("world"),
                Newline,
                Newline,
                Word("next\u{2009}line"),
            ]
        );
    }
}
