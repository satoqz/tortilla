use std::iter::Peekable;

use super::Line;

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
    upper.indent == lower.indent && upper.comment == lower.comment
    // TODO: more involved case for padding + bullet
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
        merge(vec![line!("foo", "bar")], vec![line!("foo", "bar")]);

        merge(
            vec![line!(s4, "//", s1, "-", "foo", "bar")],
            vec![line!(s4, "//", s1, "-", "foo", "bar")],
        );

        merge(
            vec![
                line!(s4, "//", s1, "-", "foo", "bar"),
                line!(t2, "//", s1, "-", "foo", "bar"),
                line!(s1, "h"),
                line!("//", s1, "comment"),
            ],
            vec![
                line!(s4, "//", s1, "-", "foo", "bar"),
                line!(t2, "//", s1, "-", "foo", "bar"),
                line!(s1, "h"),
                line!("//", s1, "comment"),
            ],
        );
    }
}
