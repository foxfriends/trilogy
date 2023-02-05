use source_span::Span;
use trilogy_scanner::Token;

pub(crate) trait Spanned {
    fn span(&self) -> Span;
}

impl Spanned for Vec<Token> {
    fn span(&self) -> Span {
        self.iter()
            .map(|token| token.span)
            .reduce(|lhs, rhs| lhs.union(rhs))
            .expect("Don't call Spanned::span() on an empty Vec<Token>")
    }
}
