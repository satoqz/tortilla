#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    Tab,
    Space,
    NewlineLF,
    NewlineCRLF,
    Word(&'a str),
}

impl Token<'_> {
    fn len(&self) -> usize {
        match self {
            Self::Tab | Self::Space | Self::NewlineLF => 1,
            Self::NewlineCRLF => 2,
            Self::Word(s) => s.len(),
        }
    }
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    if input.is_empty() {
        return tokens;
    }

    let mut pos = 0;
    let lexers = [lex_space_or_tab, lex_newline, lex_word];

    while pos < input.len() {
        let slice = &input[pos..];
        match lexers.iter().find_map(|lexer| lexer(slice)) {
            Some(token) => {
                pos += token.len();
                tokens.push(token);
            }
            None => unreachable!("no lexer matched input"),
        }
    }

    tokens
}

fn lex_space_or_tab(input: &str) -> Option<Token> {
    match input.bytes().next() {
        Some(b' ') => Some(Token::Space),
        Some(b'\t') => Some(Token::Tab),
        _ => None,
    }
}

fn lex_newline(input: &str) -> Option<Token> {
    let mut iter = input.bytes();
    match iter.next() {
        Some(b'\n') => Some(Token::NewlineLF),
        Some(b'\r') => match iter.next() {
            Some(b'\n') => Some(Token::NewlineCRLF),
            _ => None, // Lone \r is not a newline.
        },
        _ => None,
    }
}

fn lex_word(input: &str) -> Option<Token> {
    let mut iter = input.bytes().enumerate().peekable();

    while let Some((idx, byte)) = iter.next() {
        // Consume until next word break:
        if matches!(byte, b' ' | b'\t' | b'\n') {
            return Some(Token::Word(&input[..idx]));
        }

        // CRLF is slightly more complex:
        if byte == b'\r' && matches!(iter.peek(), Some((_, b'\n'))) {
            return Some(Token::Word(&input[..idx]));
        }
    }

    // The word goes until end of input.
    Some(Token::Word(input))
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
}
