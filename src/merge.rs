use std::iter::Peekable;

use unicode_width::UnicodeWidthStr;

use super::{Line, Token};

pub(super) struct Merge<'t, I>
where
    I: Iterator<Item = Line<'t>>,
{
    source: Peekable<I>,
}

pub(super) fn iter<'t, I>(source: I) -> Merge<'t, I>
where
    I: Iterator<Item = Line<'t>>,
{
    Merge {
        source: source.peekable(),
    }
}

fn should_merge(upper: &Line<'_>, lower: &Line<'_>) -> bool {
    !upper.words.is_empty() && !lower.words.is_empty() // Don't touch "empty" lines
        && lower.bullet.is_none() // Don't touch lines that start their own bullet
        && upper.comment == lower.comment // Comment token must match
        && upper.indent == lower.indent // Indent must match
        && bullet_continuation(upper, lower)
}

fn bullet_continuation(upper: &Line<'_>, lower: &Line<'_>) -> bool {
    let bullet = match upper.bullet {
        // No bullet, padding must match 1 to 1:
        None => return upper.padding == lower.padding,
        Some(bullet) => bullet,
    };

    // Bullets only work with spaces:
    if upper.padding.0 != Token::Space || lower.padding.0 != Token::Space {
        return false;
    }

    // Bullets are followed by a space that must be replicated by the lower
    // padding.
    let bullet_width = bullet.as_str().width_cjk() + 1;
    upper.padding.1 + bullet_width == lower.padding.1
}

fn merge<'t>(upper: &mut Line<'t>, mut lower: Line<'t>) {
    upper.words.append(&mut lower.words);
    upper.newline &= lower.newline;
}

impl<'t, I> Iterator for Merge<'t, I>
where
    I: Iterator<Item = Line<'t>>,
{
    type Item = Line<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut upper = self.source.next()?;

        while let Some(lower) = self.source.next_if(|lower| should_merge(&upper, lower)) {
            merge(&mut upper, lower);
        }

        Some(upper)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Line, line};

    fn merge<'t>(before: Vec<Line<'t>>, after: Vec<Line<'t>>) {
        assert_eq!(super::iter(before.into_iter()).collect::<Vec<_>>(), after);
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
