use std::collections::HashSet;

use unicode_width::UnicodeWidthStr;

use super::{Line, Newline, Toppings, Whitespace};

/// A line breaking algorithm.
pub trait Sauce {
    fn prepare(words: &[&str], max: usize) -> Self;
    fn should_break(&mut self, words: &[&str], idx: usize) -> bool;
}

/// Naive "first-fit" line breaking algorithm.
///
/// Doesn't produce optimal results, but time complexity is O(n) and space
/// complexity is O(1).
///
/// Also see: <https://en.wikipedia.org/wiki/Wrapping_(text)#Minimum_number_of_lines>
pub struct Guacamole {
    max: usize,
    width: usize,
}

/// More sophisticated "optimal-fit" line breaking algorithm.
///
/// Time complexity is O(n^2), space complexity is O(n). This is fast enough
/// for inputs of common size (i.e., reasonably sized paragraphs in a plain text
/// document or code file). This is the default algorithm used by the tortilla
/// CLI.
///
/// Also see:
/// - <https://en.wikipedia.org/wiki/Wrapping_(text)#Minimum_raggedness>
/// - <https://en.wikipedia.org/wiki/Knuth%E2%80%93Plass_line-breaking_algorithm>
pub struct Salsa(HashSet<usize>);

impl Sauce for Guacamole {
    fn prepare(_: &[&str], max: usize) -> Self {
        Self { max, width: 0 }
    }

    fn should_break(&mut self, words: &[&str], idx: usize) -> bool {
        let width = words[idx].width_cjk();

        // First word always fits, and doesn't produce an extra space.
        if self.width == 0 {
            self.width = width;
            return true;
        }

        let (updated, should_break) = match self.width {
            // First word always fits, and doesn't produce an extra space.
            0 => (width, false),
            // Add to the current line, and add a space in front.
            _ if self.width + width < self.max => (self.width + width + 1, false),
            // Start a new line first, again no need for a space.
            _ => (width, true),
        };

        self.width = updated;
        should_break
    }
}

impl Sauce for Salsa {
    fn prepare(words: &[&str], max: usize) -> Self {
        // This is shamelessly ported from:
        // https://gist.github.com/dieter-medium/ad9f47a4e7e8ef4127461771a421e614#file-shortest_path_breaks-rb

        // TODO: Maybe bother with:
        // https://www.sciencedirect.com/science/article/pii/S0166218X98000213,
        // but probably not. O(n^2) is good enough for me since I don't plan to
        // wrap megabytes of single-paragraph text... I think?

        let mut offsets = vec![0; words.len() + 1];
        for (idx, word) in words.iter().enumerate() {
            offsets[idx + 1] = offsets[idx] + word.width_cjk();
        }

        let mut minimas = vec![(0, usize::MAX); offsets.len()];
        minimas[0].1 = 0;

        for start_node_idx in 0..words.len() {
            for end_node_idx in (start_node_idx + 1)..offsets.len() {
                let line_length = offsets[end_node_idx] - offsets[start_node_idx] + end_node_idx
                    - start_node_idx
                    - 1;

                if line_length > max && end_node_idx != start_node_idx + 1 {
                    break;
                }

                let penalty = match end_node_idx != words.len() {
                    true => max.saturating_sub(line_length).pow(2),
                    false => 0,
                };

                let cost = minimas[start_node_idx].1 + penalty;
                if cost < minimas[end_node_idx].1 {
                    minimas[end_node_idx] = (start_node_idx, cost);
                }
            }
        }

        let backtrack = std::iter::successors(Some(words.len()), |idx| {
            (*idx != 0).then_some(minimas[*idx].0)
        });

        Self(backtrack.skip(1).collect())
    }

    fn should_break(&mut self, _: &[&str], idx: usize) -> bool {
        self.0.contains(&idx)
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

pub(super) struct LineWrap<'t, S> {
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
    pub fn new(line: Line<'t>, toppings: &Toppings) -> Self {
        let whitespace_width = |whitespace| match whitespace {
            Whitespace::Space(count) => count,
            Whitespace::Tab(count) => toppings.tabs * count,
        };

        let bullet_width = line
            .bullet
            .map(|bullet| bullet.width_cjk() + 1)
            .unwrap_or(0);

        let unbreakable_width = whitespace_width(line.indent)
            + line.comment.map(|comment| comment.width_cjk()).unwrap_or(0)
            + whitespace_width(line.padding)
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
    type Item = &'t str;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state {
                State::Words => {
                    if let Some(s) = self.pending.take() {
                        break Some(s);
                    }

                    let s = match self.line.words.get(self.word_idx) {
                        Some(s) => s,
                        None => {
                            self.state = State::Final;
                            break self.line.newline.then_some(self.newline.as_str());
                        }
                    };

                    let should_break = self.sauce.should_break(&self.line.words, self.word_idx);
                    self.word_idx += 1;

                    // Queue up this word:
                    self.pending = Some(s);

                    // First word special case: Start a new line, but don't
                    // prepend a newline token, rather skip right to the indent.
                    if self.word_idx == 1 {
                        self.state = State::Indent;
                        continue;
                    }

                    break Some(if should_break {
                        // Word doesn't fit, start a new line.
                        self.state = State::Indent;
                        self.newline.as_str()
                    } else {
                        // Word fits, but needs a space first.
                        self.state = State::Words;
                        " "
                    });
                }

                State::Indent if self.whitespace_idx == self.line.indent.count() => {
                    self.whitespace_idx = 0;
                    self.state = State::Comment;
                }

                State::Indent => {
                    self.whitespace_idx += 1;
                    break Some(self.line.indent.as_str());
                }

                State::Comment => {
                    self.state = State::Padding;
                    if let Some(token) = self.line.comment {
                        break Some(token);
                    }
                }

                State::Padding if self.whitespace_idx == self.line.padding.count() => {
                    self.whitespace_idx = 0;
                    self.state = State::Bullet;
                }

                State::Padding => {
                    self.whitespace_idx += 1;
                    break Some(self.line.padding.as_str());
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
                        self.whitespace_idx = self.bullet_width.saturating_sub(1);
                        break Some(token);
                    }
                }

                State::BulletSpace if self.whitespace_idx == self.bullet_width => {
                    self.whitespace_idx = 0;
                    self.state = State::Words;
                }

                State::BulletSpace => {
                    self.whitespace_idx += 1;
                    break Some(" ");
                }

                State::Final => {
                    break None;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Guacamole, Salsa};
    use crate::{Line, Toppings, Whitespace::*, line};

    fn t() -> Toppings {
        Toppings::default()
    }

    fn salsa<'t>(line: Line<'t>, toppings: &Toppings) -> Vec<&'t str> {
        super::LineWrap::<Salsa>::new(line, toppings).collect()
    }

    fn guacamole<'t>(line: Line<'t>, toppings: &Toppings) -> Vec<&'t str> {
        super::LineWrap::<Guacamole>::new(line, toppings).collect()
    }

    #[track_caller]
    fn all<'t>(line: Line<'t>, toppings: &Toppings) -> Vec<&'t str> {
        let salsa = salsa(line.clone(), toppings);
        let guacamole = guacamole(line, toppings);
        assert_eq!(salsa, guacamole);
        salsa
    }

    #[test]
    fn empty() {
        assert_eq!(
            all(line!(Space(0), None, Space(0), None), &t()),
            Vec::<&str>::new(),
        );
    }
}
