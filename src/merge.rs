use std::iter::Peekable;

use unicode_width::UnicodeWidthStr;

use super::{Line, Whitespace};

pub(super) struct Merge<L: Iterator> {
    lines: Peekable<L>,
}

impl<L: Iterator> Merge<L> {
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
    let bullet_width = bullet.width_cjk() + 1;

    // Bullets only work with space padding.
    matches!(upper_whitespace, Whitespace::Space(_))
        && matches!(lower_whitespace, Whitespace::Space(_))
        && upper_whitespace.count() + bullet_width == lower_whitespace.count()
}

fn merge<'t>(upper: &mut Line<'t>, mut lower: Line<'t>) {
    upper.words.append(&mut lower.words);
    upper.newline &= lower.newline;
}

impl<'t, L> Iterator for Merge<L>
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
    use crate::{Line, Whitespace::*, line};

    fn merge(lines: Vec<Line>) -> Vec<Line> {
        super::Merge::new(lines.into_iter()).collect()
    }

    #[test]
    fn empty() {
        assert_eq!(merge(vec![]), vec![]);
    }

    #[test]
    fn single_line() {
        assert_eq!(
            merge(vec![line!(Space(2), None, Space(2), None, "hello")]),
            vec![line!(Space(2), None, Space(2), None, "hello")]
        );
    }

    #[test]
    fn merge_two_lines_with_matching_indent_and_padding() {
        assert_eq!(
            merge(vec![
                line!(Space(2), None, Space(2), None, "hello" ;),
                line!(Space(2), None, Space(2), None, "world" ;),
            ]),
            vec![line!(Space(2), None, Space(2), None, "hello", "world" ;)]
        );
    }

    #[test]
    fn merge_two_lines_with_bullet() {
        assert_eq!(
            merge(vec![
                line!(Space(2), None, Space(2), Some("-"), "hello" ;),
                line!(Space(2), None, Space(4), None, "world"),
            ]),
            vec![line!(Space(2), None, Space(2), Some("-"), "hello", "world")]
        );
    }

    #[test]
    fn do_not_merge_different_comments() {
        assert_eq!(
            merge(vec![
                line!(Space(2), Some("//"), Space(2), None, "hello" ;),
                line!(Space(2), Some("#"), Space(2), None, "world"),
            ]),
            vec![
                line!(Space(2), Some("//"), Space(2), None, "hello" ;),
                line!(Space(2), Some("#"), Space(2), None, "world"),
            ]
        );
    }

    #[test]
    fn do_not_merge_different_bullets() {
        assert_eq!(
            merge(vec![
                line!(Space(2), None, Space(2), Some("-"), "hello" ;),
                line!(Space(2), None, Space(2), Some("*"), "world"),
            ]),
            vec![
                line!(Space(2), None, Space(2), Some("-"), "hello" ;),
                line!(Space(2), None, Space(2), Some("*"), "world"),
            ]
        );
    }

    #[test]
    fn do_not_merge_empty_lines() {
        assert_eq!(
            merge(vec![
                line!(Space(2), None, Space(2), None, "hello" ;),
                line!(Space(2), None, Space(2), None ;),
                line!(Space(2), None, Space(2), None, "world" ;),
            ]),
            vec![
                line!(Space(2), None, Space(2), None, "hello" ;),
                line!(Space(2), None, Space(2), None ;),
                line!(Space(2), None, Space(2), None, "world" ;),
            ]
        );
    }

    #[test]
    fn do_not_merge_lines_with_new_bullet() {
        assert_eq!(
            merge(vec![
                line!(Space(2), None, Space(2), Some("-"), "hello" ;),
                line!(Space(2), None, Space(2), Some("-"), "world" ;),
            ]),
            vec![
                line!(Space(2), None, Space(2), Some("-"), "hello" ;),
                line!(Space(2), None, Space(2), Some("-"), "world" ;),
            ]
        );
    }

    #[test]
    fn merge_multiple_lines() {
        assert_eq!(
            merge(vec![
                line!(Space(2), None, Space(2), None, "hello" ;),
                line!(Space(2), None, Space(2), None, "world" ;),
                line!(Space(2), None, Space(2), None, "rust"),
            ]),
            vec![line!(
                Space(2),
                None,
                Space(2),
                None,
                "hello",
                "world",
                "rust"
            )]
        );
    }
}
