use source_span::Span;
use trilogy_scanner::Token;

/// Provides access to the [`Span`][] that this piece of source code takes up
/// in the source file.
pub trait Spanned {
    fn span(&self) -> Span;
}

impl<S> Spanned for Vec<S>
where
    S: Spanned,
{
    fn span(&self) -> Span {
        self.iter()
            .map(|el| el.span())
            .reduce(|lhs, rhs| lhs.union(rhs))
            .expect("Don't call Spanned::span() on an empty Vec<S>")
    }
}

impl<S> Spanned for Box<S>
where
    S: Spanned,
{
    fn span(&self) -> Span {
        (**self).span()
    }
}

impl Spanned for Token {
    fn span(&self) -> Span {
        self.span
    }
}
