use unicode_width::UnicodeWidthStr;

use super::{Line, Newline, Token, Toppings};

enum Fit {
    First,
    This,
    Next,
}

trait Strategy {
    fn fit(&mut self, width: usize) -> Fit;
}

struct NaiveStrategy {
    max: usize,
    width: usize,
}

impl NaiveStrategy {
    fn new(max: usize) -> Self {
        Self { max, width: 0 }
    }
}

impl Strategy for NaiveStrategy {
    fn fit(&mut self, width: usize) -> Fit {
        if self.width == 0 {
            self.width = width;
            Fit::First
        } else if self.width + width + 1 <= self.max {
            self.width += width + 1;
            Fit::This
        } else {
            self.width = width;
            Fit::Next
        }
    }
}

#[derive(Debug)]
enum State {
    Indent,
    Comment,
    Padding,
    Bullet,
    BulletSpace,
    Words,
    WordSpace,
    Final,
}

struct LineWrap<'t, S: Strategy> {
    line: Line<'t>,
    strategy: S,
    state: State,
    newline: Newline,
    pending: Option<&'t str>,
    word_idx: usize,
    whitespace_idx: usize,
    bullet_width: usize,
}

impl<'t> LineWrap<'t, NaiveStrategy> {
    fn new(line: Line<'t>, toppings: &Toppings) -> Self {
        let width = |token| match token {
            Token::Space => 1,
            Token::Tab => toppings.tabs,
            Token::Newline(_) => 0,
            Token::Word(s) => s.width_cjk(),
        };

        let bullet_width = line.bullet.map(width).unwrap_or(0);

        let unbreakable_width = width(line.indent.0) * line.indent.1
            + line.comment.map(width).unwrap_or(0)
            + width(line.padding.0) * line.padding.1
            + bullet_width;

        let breakable_width = toppings.width.saturating_sub(unbreakable_width);

        Self {
            line,
            strategy: NaiveStrategy::new(breakable_width),
            newline: toppings.newline,
            state: State::Indent,
            pending: None,
            word_idx: 0,
            whitespace_idx: 0,
            bullet_width,
        }
    }
}

impl<'t, S: Strategy> Iterator for LineWrap<'t, S> {
    type Item = Token<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state {
                State::Indent if self.whitespace_idx == self.line.indent.1 => {
                    self.whitespace_idx = 0;
                    self.state = State::Comment;
                }

                State::Indent => {
                    self.whitespace_idx += 1;
                    break Some(self.line.indent.0);
                }

                State::Comment => {
                    self.state = State::Padding;
                    if let Some(token) = self.line.comment {
                        break Some(token);
                    }
                }

                State::Padding if self.whitespace_idx == self.line.padding.1 => {
                    self.whitespace_idx = 0;
                    self.state = State::Bullet;
                }

                State::Padding => {
                    self.whitespace_idx += 1;
                    break Some(self.line.padding.0);
                }

                State::Bullet => {
                    let token = match self.line.bullet {
                        Some(token) => token,
                        None => {
                            self.state = State::Words;
                            continue;
                        }
                    };

                    if self.pending.is_some() {
                        self.state = State::BulletSpace;
                        continue;
                    }

                    self.state = State::WordSpace;
                    break Some(token);
                }

                State::BulletSpace if self.whitespace_idx == self.bullet_width => {
                    self.whitespace_idx = 0;
                    self.state = State::WordSpace;
                }

                State::BulletSpace => {
                    self.whitespace_idx += 1;
                    break Some(Token::Space);
                }

                State::WordSpace => {
                    self.state = State::Words;
                    break Some(Token::Space);
                }

                State::Words => {
                    if let Some(s) = self.pending.take() {
                        break Some(Token::Word(s));
                    }

                    let s = match self.line.words.get(self.word_idx) {
                        Some(s) => s,
                        None => {
                            self.state = State::Final;
                            break self.line.newline.then_some(Token::Newline(self.newline));
                        }
                    };

                    self.word_idx += 1;

                    match self.strategy.fit(s.width_cjk()) {
                        Fit::First => {
                            break Some(Token::Word(s));
                        }
                        Fit::This => {
                            self.state = State::Words;
                            self.pending = Some(s);
                            break Some(Token::Space);
                        }
                        Fit::Next => {
                            self.state = State::Indent;
                            self.pending = Some(s);
                            break Some(Token::Newline(self.newline));
                        }
                    }
                }

                State::Final => {
                    break None;
                }
            }
        }
    }
}

pub(super) struct Wrap<'t, I> {
    source: I,
    toppings: Toppings,
    inner: Option<LineWrap<'t, NaiveStrategy>>,
}

pub(super) fn iter<'t, I>(source: I, toppings: Toppings) -> Wrap<'t, I>
where
    I: Iterator<Item = Line<'t>>,
{
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
                    .insert(LineWrap::new(self.source.next()?, &self.toppings)),
            };

            match inner.next() {
                Some(token) => return Some(token),
                None => self.inner = None,
            }
        }
    }
}
