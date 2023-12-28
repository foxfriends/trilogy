use std::iter::Peekable;

pub trait PairableIterator {
    type Item;
    type Iterator: Iterator<Item = (Self::Item, Self::Item)>;

    fn pairwise(self) -> Self::Iterator;
}

pub struct Pairwise<T>(Peekable<T>)
where
    T: Iterator;

impl<T> PairableIterator for Peekable<T>
where
    T: Iterator,
    T::Item: Copy,
{
    type Item = T::Item;
    type Iterator = Pairwise<T>;

    fn pairwise(self) -> Pairwise<T> {
        Pairwise(self)
    }
}

impl<T> ExactSizeIterator for Pairwise<T>
where
    T: Iterator + ExactSizeIterator,
    T::Item: Copy,
{
    fn len(&self) -> usize {
        self.0.len().saturating_sub(1)
    }
}

impl<T> Iterator for Pairwise<T>
where
    T: Iterator,
    T::Item: Copy,
{
    type Item = (T::Item, T::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let this = self.0.next()?;
        let next = self.0.peek()?;
        Some((this, *next))
    }
}
