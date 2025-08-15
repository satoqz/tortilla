use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    /// ' '
    Space,
    /// '\t'
    Tab,
    /// '\n'
    NewlineLF,
    /// '\r\n'
    NewlineCRLF,
    /// A chain of characters without any whitespace.
    Word(&'a str),
}

fn word_break(grapheme: &str) -> Option<Token<'static>> {
    Some(match grapheme {
        " " => Token::Space,
        "\t" => Token::Tab,
        "\n" => Token::NewlineLF,
        "\r\n" => Token::NewlineCRLF,
        _ => return None,
    })
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    enum State {
        Clean,
        Word(usize),
    }

    let mut state = State::Clean;

    for (byte_idx, grapheme) in input.grapheme_indices(true) {
        match state {
            State::Clean => {
                if let Some(token) = word_break(grapheme) {
                    tokens.push(token);
                } else {
                    state = State::Word(byte_idx);
                }
            }

            State::Word(start_idx) => {
                if let Some(token) = word_break(grapheme) {
                    tokens.push(Token::Word(&input[start_idx..byte_idx]));
                    tokens.push(token);
                    state = State::Clean;
                }
            }
        }
    }

    if let State::Word(start_idx) = state {
        tokens.push(Token::Word(&input[start_idx..]));
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        assert_eq!(lex(""), vec![]);
    }

    #[test]
    fn test_single_space() {
        assert_eq!(lex(" "), vec![Token::Space]);
    }

    #[test]
    fn test_single_tab() {
        assert_eq!(lex("\t"), vec![Token::Tab]);
    }

    #[test]
    fn test_single_lf() {
        assert_eq!(lex("\n"), vec![Token::NewlineLF]);
    }

    #[test]
    fn test_single_crlf() {
        assert_eq!(lex("\r\n"), vec![Token::NewlineCRLF]);
    }

    #[test]
    fn test_single_word() {
        assert_eq!(lex("hello"), vec![Token::Word("hello")]);
    }

    #[test]
    fn test_mixed_tokens() {
        assert_eq!(
            lex("hello \tworld\n\r\nnext\tline"),
            vec![
                Token::Word("hello"),
                Token::Space,
                Token::Tab,
                Token::Word("world"),
                Token::NewlineLF,
                Token::NewlineCRLF,
                Token::Word("next"),
                Token::Tab,
                Token::Word("line"),
            ]
        );
    }

    #[test]
    fn test_multiple_spaces() {
        assert_eq!(lex("   "), vec![Token::Space, Token::Space, Token::Space]);
    }

    #[test]
    fn test_multiple_tabs() {
        assert_eq!(lex("\t\t\t"), vec![Token::Tab, Token::Tab, Token::Tab]);
    }

    #[test]
    fn test_multiple_newlines() {
        assert_eq!(
            lex("\n\n\r\n"),
            vec![Token::NewlineLF, Token::NewlineLF, Token::NewlineCRLF,]
        );
    }

    #[test]
    fn test_word_with_spaces_and_tabs() {
        assert_eq!(
            lex("a b\tc\nd"),
            vec![
                Token::Word("a"),
                Token::Space,
                Token::Word("b"),
                Token::Tab,
                Token::Word("c"),
                Token::NewlineLF,
                Token::Word("d"),
            ]
        );
    }

    #[test]
    fn test_word_with_mixed_newlines() {
        assert_eq!(
            lex("a\nb\r\nc"),
            vec![
                Token::Word("a"),
                Token::NewlineLF,
                Token::Word("b"),
                Token::NewlineCRLF,
                Token::Word("c"),
            ]
        );
    }

    #[test]
    fn test_only_newlines() {
        assert_eq!(
            lex("\n\r\n\n"),
            vec![Token::NewlineLF, Token::NewlineCRLF, Token::NewlineLF,]
        );
    }

    #[test]
    fn test_lone_cr_in_word() {
        assert_eq!(lex("a\rb"), vec![Token::Word("a\rb")]);
    }

    #[test]
    fn test_cr_not_followed_by_lf() {
        assert_eq!(lex("a\r"), vec![Token::Word("a\r")]);
    }

    #[test]
    fn test_crlf_as_newline() {
        assert_eq!(
            lex("a\r\nb"),
            vec![Token::Word("a"), Token::NewlineCRLF, Token::Word("b")]
        );
    }

    #[test]
    fn test_emoji() {
        assert_eq!(lex("hello🇩🇪world"), vec![Token::Word("hello🇩🇪world")]);
    }

    #[test]
    fn test_combining_mark() {
        assert_eq!(lex("café"), vec![Token::Word("café")]);
    }

    #[test]
    fn test_multibyte_chars() {
        assert_eq!(
            lex("汉字 test"),
            vec![Token::Word("汉字"), Token::Space, Token::Word("test")]
        );
    }

    #[test]
    fn test_emoji_with_spaces() {
        assert_eq!(
            lex("hello 🇩🇪 world"),
            vec![
                Token::Word("hello"),
                Token::Space,
                Token::Word("🇩🇪"),
                Token::Space,
                Token::Word("world"),
            ]
        );
    }

    #[test]
    fn test_complex_grapheme_clusters() {
        assert_eq!(
            lex("Zöe\tétoile\n\r\n👨‍👩‍👧‍👦"),
            vec![
                Token::Word("Zöe"),
                Token::Tab,
                Token::Word("étoile"),
                Token::NewlineLF,
                Token::NewlineCRLF,
                Token::Word("👨‍👩‍👧‍👦"),
            ]
        );
    }

    #[test]
    fn test_nbsp_as_part_of_word() {
        assert_eq!(
            lex("hello\u{A0}world"),
            vec![Token::Word("hello\u{A0}world")]
        );
    }

    #[test]
    fn test_mixed_unicode_and_ascii() {
        assert_eq!(
            lex("hello\u{A0}🌍\tworld\n\r\nnext\u{2009}line"),
            vec![
                Token::Word("hello\u{A0}🌍"),
                Token::Tab,
                Token::Word("world"),
                Token::NewlineLF,
                Token::NewlineCRLF,
                Token::Word("next\u{2009}line"),
            ]
        );
    }
}
