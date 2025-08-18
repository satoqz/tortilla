use super::Line;

pub(super) struct Merge<I> {
    source: I,
}

pub(super) fn iter<'t, I>(source: I) -> Merge<I> {
    Merge { source }
}

impl<'t, I> Iterator for Merge<I>
where
    I: Iterator<Item = Line<'t>>,
{
    type Item = Line<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
