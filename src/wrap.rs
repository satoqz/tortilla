use unicode_width::UnicodeWidthStr;

use super::{Line, Newline, Token, Toppings};

pub enum Fit {
    First,
    This,
    Next,
}

pub trait Sauce {
    fn new(words: &[&str], max: usize) -> Self;
    fn fit(&mut self, words: &[&str], idx: usize) -> Fit;
}

pub struct Naive {
    max: usize,
    width: usize,
}

impl Sauce for Naive {
    fn new(_: &[&str], max: usize) -> Self {
        Self { max, width: 0 }
    }

    fn fit(&mut self, words: &[&str], idx: usize) -> Fit {
        let width = words[idx].width_cjk();

        let (updated, fit) = match self.width {
            0 => (width, Fit::First),
            // < and not <= to leave room for a space before the word.
            _ if self.width + width < self.max => (self.width + width + 1, Fit::This),
            _ => (width, Fit::Next),
        };

        self.width = updated;
        fit
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

struct LineWrap<'t, S: Sauce> {
    line: Line<'t>,
    sauce: S,
    state: State,
    newline: Newline,
    pending: Option<&'t str>,
    word_idx: usize,
    whitespace_idx: usize,
    bullet_width: usize,
}

impl<'t, S: Sauce> LineWrap<'t, S> {
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
        let strategy = S::new(&line.words, breakable_width);

        Self {
            line,
            sauce: strategy,
            newline: toppings.newline,
            state: State::Indent,
            pending: None,
            word_idx: 0,
            whitespace_idx: 0,
            bullet_width,
        }
    }
}

impl<'t, S: Sauce> Iterator for LineWrap<'t, S> {
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

                    let token = match self.sauce.fit(&self.line.words, self.word_idx) {
                        Fit::First => Token::Word(s),
                        Fit::This => {
                            self.state = State::Words;
                            self.pending = Some(s);
                            Token::Space
                        }
                        Fit::Next => {
                            self.state = State::Indent;
                            self.pending = Some(s);
                            Token::Newline(self.newline)
                        }
                    };

                    self.word_idx += 1;
                    break Some(token);
                }

                State::Final => {
                    break None;
                }
            }
        }
    }
}

pub(super) struct Wrap<'t, L, S: Sauce> {
    lines: L,
    toppings: Toppings,
    inner: Option<LineWrap<'t, S>>,
}

impl<'t, L, S> Wrap<'t, L, S>
where
    L: Iterator<Item = Line<'t>>,
    S: Sauce,
{
    pub fn new(lines: L, toppings: Toppings) -> Self {
        Self {
            lines,
            toppings,
            inner: None,
        }
    }
}

impl<'t, I, S> Iterator for Wrap<'t, I, S>
where
    I: Iterator<Item = Line<'t>>,
    S: Sauce,
{
    type Item = Token<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let inner = match &mut self.inner {
                Some(inner) => inner,
                None => self
                    .inner
                    .insert(LineWrap::new(self.lines.next()?, &self.toppings)),
            };

            match inner.next() {
                Some(token) => return Some(token),
                None => self.inner = None,
            }
        }
    }
}
