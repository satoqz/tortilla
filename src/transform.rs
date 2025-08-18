use unicode_width::UnicodeWidthStr;

use super::{Options, Token};

pub struct Transformer<'t, I> {
    source: I,
    options: Options,

    line_width: usize,
    newline_count: usize,
    pending_token: Option<Token<'t>>,
}

pub fn transform<'t, I>(source: I, options: Options) -> Transformer<'t, I>
where
    I: Iterator<Item = Token<'t>>,
{
    Transformer {
        source,
        options,
        line_width: 0,
        newline_count: 0,
        pending_token: None,
    }
}

impl<'t, I> Iterator for Transformer<'t, I>
where
    I: Iterator<Item = Token<'t>>,
{
    type Item = Token<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token) = self.pending_token.take() {
            return Some(token);
        }

        while let Some(token) = self.source.next() {
            match token {
                Token::Space | Token::Tab => self.newline_count = 0,
                Token::Newline => {
                    self.newline_count += 1;

                    if self.newline_count == 1 && self.line_width != 0 {
                        // Swallow initial newline, we start respecting newlines
                        // in the input only once there are two consecutive
                        // ones.
                        continue;
                    }

                    if self.newline_count == 2 {
                        // "Catch up" after swallowing the initial newline.
                        self.pending_token = Some(Token::Newline);
                    }

                    self.line_width = 0;
                    return Some(Token::Newline);
                }
                Token::Word(s) => {
                    self.newline_count = 0;
                    let width = s.width_cjk();

                    if self.line_width == 0 {
                        self.line_width = width;
                        return Some(Token::Word(s));
                    }

                    self.pending_token = Some(token);

                    // How long would the current line be if we added a space +
                    // this word?
                    let next_width = self.line_width + 1 + width;

                    // Word fits into line:
                    if next_width <= self.options.line_width {
                        self.line_width = next_width;
                        return Some(Token::Space);
                    }

                    // Start a new line:
                    self.line_width = width;
                    return Some(Token::Newline);
                }
            }
        }

        let final_newline =
            (self.newline_count == 1 && self.line_width != 0).then_some(Token::Newline);
        self.newline_count = 0;
        final_newline
    }
}

#[cfg(test)]
mod tests {
    use super::Options;
    use super::Token::{self, *};

    fn transform(input: Vec<Token>) -> Vec<Token> {
        super::transform(input.into_iter(), Options::default()).collect()
    }

    #[test]
    fn what() {
        assert_eq!(transform(vec![Newline]), vec![Newline]);

        assert_eq!(
            transform(vec![Word("bruh"), Newline, Word("bruh")]),
            vec![Word("bruh"), Space, Word("bruh")]
        );

        assert_eq!(
            transform(vec![Word("bruh"), Newline, Newline, Word("bruh")]),
            vec![Word("bruh"), Newline, Newline, Word("bruh")]
        );
    }
}
