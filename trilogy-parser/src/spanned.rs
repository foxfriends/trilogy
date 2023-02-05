use source_span::Span;
use trilogy_scanner::Token;

pub(crate) trait Spanned {
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

impl Spanned for Token {
    fn span(&self) -> Span {
        self.span
    }
}
