use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Pack {
    pub values: Vec<Element>,
}

#[derive(Clone, Debug)]
pub struct Element {
    pub span: Span,
    pub expression: Expression,
    pub is_spread: bool,
}

impl From<Expression> for Element {
    fn from(expression: Expression) -> Self {
        Self {
            span: expression.span,
            expression,
            is_spread: false,
        }
    }
}

impl FromIterator<Expression> for Pack {
    fn from_iter<T: IntoIterator<Item = Expression>>(iter: T) -> Self {
        let values: Vec<_> = iter.into_iter().map(Element::from).collect();
        Self { values }
    }
}
