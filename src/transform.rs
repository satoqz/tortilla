use super::Token;

pub struct Options {
    /// Maximum line width to wrap at.
    pub line_width: usize,
    /// How much a tab indent contributes to line width.
    pub tab_width: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            line_width: 80,
            tab_width: 4,
        }
    }
}

pub struct Transformer<'a, I> {
    source: I,
    options: Options,
    last: Option<Token<'a>>,
}

pub fn transform<'a, I>(source: I, options: Options) -> Transformer<'a, I>
where
    I: Iterator<Item = Token<'a>>,
{
    Transformer {
        source,
        options,
        last: None,
    }
}

impl<'a, I: Iterator<Item = Token<'a>>> Iterator for Transformer<'a, I> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let prev = self.last;
        self.last = self.source.next();

        if matches!((prev, self.last), (Some(Token::Space), Some(Token::Space))) {
            return self.next();
        }

        self.last
    }
}
