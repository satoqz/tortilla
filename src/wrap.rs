use unicode_width::UnicodeWidthStr;

use super::{Line, Token, Toppings};

enum Section {
    Indent,
    Comment,
    Padding,
    Bullet,
    Words,
    Space,
    Newline,
    Final,
}

struct LineWrap<'t> {
    line: Line<'t>,
    toppings: Toppings,
    section: Section,
    width: usize,
    word: usize,
}

impl<'t> LineWrap<'t> {
    fn new(line: Line<'t>, toppings: Toppings) -> Self {
        Self {
            line,
            toppings,
            section: Section::Indent,
            width: 0,
            word: 0,
        }
    }
}

impl<'t, 'idk> Iterator for LineWrap<'t> {
    type Item = Token<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.section {
                Section::Indent if self.line.indent.1 == 0 => {
                    self.section = Section::Comment;
                }

                Section::Indent => {
                    self.line.indent.1 -= 1;
                    break Some(self.line.indent.0);
                }

                Section::Comment => {
                    self.section = Section::Padding;
                    if let Some(token) = self.line.comment {
                        break Some(token);
                    }
                }

                Section::Padding if self.line.padding.1 == 0 => {
                    self.section = Section::Bullet;
                }

                Section::Padding => {
                    self.line.padding.1 -= 1;
                    break Some(self.line.padding.0);
                }

                Section::Bullet => {
                    if let Some(token) = self.line.bullet {
                        self.section = Section::Space;
                        break Some(token);
                    } else {
                        self.section = Section::Words;
                    }
                }

                Section::Space => {
                    self.section = Section::Words;
                    break Some(Token::Space);
                }

                Section::Words => {
                    if let Some(s) = self.line.words.get(self.word) {
                        let next_width = self.width + s.width_cjk();

                        if next_width > self.toppings.width {
                            self.section = Section::Newline;
                            continue;
                        }

                        self.word += 1;

                        if self.word == self.line.words.len() {
                            self.section = Section::Words;
                            break Some(Token::Word(s));
                        }

                        if next_width + 1 > self.toppings.width {
                            self.section = Section::Newline;
                        } else {
                            self.section = Section::Space;
                        }

                        break Some(Token::Word(s));
                    } else {
                        self.section = Section::Final;
                        break self
                            .line
                            .newline
                            .then_some(Token::Newline(self.toppings.newline));
                    }
                }

                Section::Newline => {
                    self.section = Section::Indent;
                    self.width = 0;
                    break Some(Token::Newline(self.toppings.newline));
                }

                Section::Final => {
                    break None;
                }
            }
        }
        .inspect(|token| self.width += self.token_width(token))
    }
}

impl<'t> LineWrap<'t> {
    fn token_width(&self, token: &Token<'t>) -> usize {
        match token {
            Token::Space => 1,
            Token::Tab => self.toppings.tabs,
            Token::Newline(_) => 0,
            Token::Word(s) => s.width_cjk(),
        }
    }
}

pub(super) struct Wrap<'t, I> {
    source: I,
    toppings: Toppings,
    inner: Option<LineWrap<'t>>,
}

pub(super) fn iter<'t, I>(source: I, toppings: Toppings) -> Wrap<'t, I> {
    Wrap {
        source,
        toppings,
        inner: None,
    }
}

impl<'t, I> Iterator for Wrap<'t, I>
where
    I: Iterator<Item = Line<'t>>,
{
    type Item = Token<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let inner = match &mut self.inner {
                Some(inner) => inner,
                None => self
                    .inner
                    .insert(LineWrap::new(self.source.next()?, self.toppings.clone())),
            };

            match inner.next() {
                Some(token) => return Some(token),
                None => self.inner = None,
            }
        }
    }
}
