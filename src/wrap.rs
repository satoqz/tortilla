use unicode_width::UnicodeWidthStr;

use super::{Line, Token, Toppings};

enum Section {
    Indent(usize),
    Comment,
    Padding(usize),
    Bullet,
    Words(usize),
    Space(usize),
    Final,
}

struct State<'t> {
    line: Line<'t>,
    width: usize,
    section: Section,
}

pub(super) struct Wrap<'t, I> {
    source: I,
    state: Option<State<'t>>,
    toppings: Toppings,
}

pub(super) fn iter<'t, I>(source: I, toppings: Toppings) -> Wrap<'t, I> {
    Wrap {
        source,
        toppings,
        state: None,
    }
}

impl<'t, I> Iterator for Wrap<'t, I>
where
    I: Iterator<Item = Line<'t>>,
{
    type Item = Token<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state.take() {
                None => {
                    self.state = Some(State {
                        line: self.source.next()?,
                        width: 0,
                        section: Section::Indent(0),
                    })
                }

                Some(State {
                    line,
                    width,
                    section,
                }) => match section {
                    Section::Indent(idx) => {
                        if idx < line.indent.1 {
                            let token = line.indent.0;
                            self.state = Some(State {
                                line,
                                width: width + self.token_width(token),
                                section: Section::Indent(idx + 1),
                            });
                            return Some(token);
                        } else {
                            self.state = Some(State {
                                line,
                                width,
                                section: Section::Comment,
                            });
                        }
                    }

                    Section::Comment => {
                        if let Some(token) = line.comment {
                            self.state = Some(State {
                                line,
                                width: self.token_width(token),
                                section: Section::Padding(0),
                            });
                            return Some(token);
                        } else {
                            self.state = Some(State {
                                line,
                                width,
                                section: Section::Padding(0),
                            });
                        }
                    }

                    Section::Padding(idx) => {
                        if idx < line.padding.1 {
                            let token = line.padding.0;
                            self.state = Some(State {
                                line,
                                width: width + self.token_width(token),
                                section: Section::Padding(idx + 1),
                            });
                            return Some(token);
                        } else {
                            self.state = Some(State {
                                line,
                                width,
                                section: Section::Bullet,
                            });
                        }
                    }

                    Section::Bullet => {
                        if let Some(token) = line.bullet {
                            let section = if line.words.is_empty() {
                                Section::Final
                            } else {
                                Section::Space(0)
                            };
                            self.state = Some(State {
                                line,
                                width: width + self.token_width(token),
                                section,
                            });
                            return Some(token);
                        } else {
                            self.state = Some(State {
                                line,
                                width,
                                section: Section::Words(0),
                            });
                        }
                    }

                    Section::Space(idx) => {
                        self.state = Some(State {
                            line,
                            width: width + 1,
                            section: Section::Words(idx),
                        });
                        return Some(Token::Space);
                    }

                    Section::Words(idx) => {
                        if let Some(word) = line.words.get(idx) {
                            let token = Token::Word(*word);
                            let section = if line.words.len() == idx + 1 {
                                Section::Final
                            } else {
                                Section::Space(idx + 1)
                            };
                            self.state = Some(State {
                                line,
                                width: width + self.token_width(token),
                                section,
                            });
                            return Some(token);
                        } else {
                            self.state = Some(State {
                                line,
                                width,
                                section: Section::Final,
                            });
                        }
                    }

                    Section::Final => {
                        self.state = None;
                        if line.newline {
                            return Some(Token::Newline(self.toppings.newline));
                        }
                    }
                },
            }
        }
    }
}

impl<'t, I> Wrap<'t, I>
where
    I: Iterator<Item = Line<'t>>,
{
    fn token_width(&self, token: Token<'t>) -> usize {
        match token {
            Token::Space | Token::Newline(_) => 1,
            Token::Tab => self.toppings.tab_size,
            Token::Word(s) => s.width_cjk(),
        }
    }
}
