use unicode_width::UnicodeWidthStr;

use super::{Line, Newline, Token, Toppings};

pub trait Sauce {
    fn prepare(words: &[&str], max: usize) -> Self;
    fn fits(&mut self, words: &[&str], idx: usize) -> bool;
}

pub struct Guacamole {
    max: usize,
    width: usize,
}

impl Sauce for Guacamole {
    fn prepare(_: &[&str], max: usize) -> Self {
        Self { max, width: 0 }
    }

    fn fits(&mut self, words: &[&str], idx: usize) -> bool {
        let width = words[idx].width_cjk();

        // First word always fits, and doesn't produce an extra space.
        if self.width == 0 {
            self.width = width;
            return true;
        }

        let (updated, fits) = match self.width {
            // First word always fits, and doesn't produce an extra space.
            0 => (width, true),
            // Add to the current line, and add a space in front.
            _ if self.width + width < self.max => (self.width + width + 1, true),
            // Start a new line first, again no need for a space.
            _ => (width, false),
        };

        self.width = updated;
        fits
    }
}

pub struct Salsa;

impl Sauce for Salsa {
    fn prepare(_: &[&str], _: usize) -> Self {
        Self
    }

    fn fits(&mut self, _: &[&str], _: usize) -> bool {
        unimplemented!("salsa is still in preparation")
    }
}

#[derive(Debug)]
enum State {
    Words,
    Indent,
    Comment,
    Padding,
    Bullet,
    BulletSpace,
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
        let sauce = S::prepare(&line.words, breakable_width);

        let state = if line.words.is_empty() {
            State::Indent
        } else {
            State::Words
        };

        Self {
            line,
            sauce,
            state,
            newline: toppings.newline,
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

                    let fits = self.sauce.fits(&self.line.words, self.word_idx);
                    self.word_idx += 1;

                    // Queue up this word:
                    self.pending = Some(s);

                    // First word special case: Start a new line, but don't
                    // prepend a newline token, rather skip right to the indent.
                    if self.word_idx == 1 {
                        self.state = State::Indent;
                        continue;
                    }

                    break Some(if fits {
                        // Word fits, but needs a space first.
                        self.state = State::Words;
                        Token::Space
                    } else {
                        // Word doesn't fit, start a new line.
                        self.state = State::Indent;
                        Token::Newline(self.newline)
                    });
                }

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

                    // No more words come after this bullet, don't insert space.
                    if self.pending.is_none() {
                        // Go back to words to finalize the line.
                        self.state = State::Words;
                        break Some(token);
                    }

                    self.state = State::BulletSpace;

                    if self.word_idx == 1 {
                        // Only add a single space, after the bullet.
                        self.whitespace_idx = self.bullet_width;
                        break Some(token);
                    }
                }

                State::BulletSpace if self.whitespace_idx == self.bullet_width + 1 => {
                    self.whitespace_idx = 0;
                    self.state = State::Words;
                }

                State::BulletSpace => {
                    self.whitespace_idx += 1;
                    break Some(Token::Space);
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
