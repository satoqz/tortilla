use std::iter::Peekable;

use unicode_width::UnicodeWidthStr;

use super::{Line, Token};

pub(super) struct Merge<'t, L>
where
    L: Iterator<Item = Line<'t>>,
{
    lines: Peekable<L>,
}

impl<'t, L> Merge<'t, L>
where
    L: Iterator<Item = Line<'t>>,
{
    pub fn new(lines: L) -> Self {
        Self {
            lines: lines.peekable(),
        }
    }
}

fn should_merge(upper: &Line<'_>, lower: &Line<'_>) -> bool {
    !upper.words.is_empty() && !lower.words.is_empty() // Don't touch "empty" lines
        && lower.bullet.is_none() // Don't touch lines that start their own bullet
        && upper.comment == lower.comment // Comment token must match
        && bullet_continuation(upper, lower)
}

fn bullet_continuation(upper: &Line<'_>, lower: &Line<'_>) -> bool {
    let bullet = match upper.bullet {
        // No bullet, padding and indent must match 1 to 1:
        None => return upper.padding == lower.padding && upper.indent == lower.indent,
        Some(bullet) => bullet,
    };

    // If indents are equal, we only need to check the padding:
    let (upper_whitespace, lower_whitespace) = match upper.indent == lower.indent {
        true => (&upper.padding, &lower.padding),
        false => (&upper.indent, &lower.indent),
    };

    // +1 for space between bullet and word.
    let bullet_width = bullet.as_str().width_cjk() + 1;

    // Bullets only work with space padding.
    upper_whitespace.0 == Token::Space
        && lower_whitespace.0 == Token::Space
        && upper_whitespace.1 + bullet_width == lower_whitespace.1
}

fn merge<'t>(upper: &mut Line<'t>, mut lower: Line<'t>) {
    upper.words.append(&mut lower.words);
    upper.newline &= lower.newline;
}

impl<'t, L> Iterator for Merge<'t, L>
where
    L: Iterator<Item = Line<'t>>,
{
    type Item = Line<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut upper = self.lines.next()?;

        while let Some(lower) = self.lines.next_if(|lower| should_merge(&upper, lower)) {
            merge(&mut upper, lower);
        }

        Some(upper)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Line, line};

    fn merge<'t>(before: Vec<Line<'t>>, after: Vec<Line<'t>>) {
        assert_eq!(
            super::Merge::new(before.into_iter()).collect::<Vec<_>>(),
            after
        );
    }

    #[test]
    fn foo() {
        merge(
            vec![line!(s0, "", s0, "", "foo", "bar")],
            vec![line!(s0, "", s0, "", "foo", "bar")],
        );

        merge(
            vec![line!(s4, "//", s1, "-", "foo", "bar")],
            vec![line!(s4, "//", s1, "-", "foo", "bar")],
        );

        merge(
            vec![
                line!(s4, "//", s1, "-", "foo", "bar"),
                line!(s4, "//", s3, "", "baz"),
                line!(s4, "//", s1, "", "nope"),
            ],
            vec![
                line!(s4, "//", s1, "-", "foo", "bar", "baz"),
                line!(s4, "//", s1, "", "nope"),
            ],
        );
    }
}
