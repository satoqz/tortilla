use super::{Line, Token, Toppings};

pub(super) struct Wrap<I> {
    source: I,
    toppings: Toppings,
}

pub(super) fn iter<'t, I>(source: I, toppings: Toppings) -> Wrap<I> {
    Wrap { source, toppings }
}

impl<'t, I> Iterator for Wrap<I>
where
    I: Iterator<Item = Line<'t>>,
{
    type Item = Token<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
