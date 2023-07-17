use super::*;
use crate::{Analyzer, Id};
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Pack {
    pub values: Vec<Element>,
}

impl Pack {
    pub fn len(&self) -> Option<usize> {
        if self.values.iter().all(|val| !val.is_spread) {
            Some(self.values.len())
        } else {
            None
        }
    }

    pub(super) fn bindings(&self) -> impl std::iter::Iterator<Item = Id> + '_ {
        self.values.iter().flat_map(|val| val.expression.bindings())
    }
}

#[derive(Clone, Debug)]
pub struct Element {
    pub span: Span,
    pub expression: Expression,
    pub is_spread: bool,
}

impl Element {
    pub(super) fn convert_array(analyzer: &mut Analyzer, ast: syntax::ArrayElement) -> Self {
        match ast {
            syntax::ArrayElement::Element(ast) => {
                let expression = Expression::convert(analyzer, ast);
                Self::from(expression)
            }
            syntax::ArrayElement::Spread(token, ast) => {
                let span = ast.span().union(token.span);
                let expression = Expression::convert(analyzer, ast);
                Self {
                    span,
                    expression,
                    is_spread: true,
                }
            }
        }
    }

    pub(super) fn convert_set(analyzer: &mut Analyzer, ast: syntax::SetElement) -> Self {
        match ast {
            syntax::SetElement::Element(ast) => {
                let expression = Expression::convert(analyzer, ast);
                Self::from(expression)
            }
            syntax::SetElement::Spread(token, ast) => {
                let span = ast.span().union(token.span);
                let expression = Expression::convert(analyzer, ast);
                Self {
                    span,
                    expression,
                    is_spread: true,
                }
            }
        }
    }

    pub(super) fn convert_record(analyzer: &mut Analyzer, ast: syntax::RecordElement) -> Self {
        match ast {
            syntax::RecordElement::Element(key, value) => {
                let key = Expression::convert(analyzer, key);
                let value = Expression::convert(analyzer, value);
                let expression = Expression::mapping(key.span.union(value.span), key, value);
                Self::from(expression)
            }
            syntax::RecordElement::Spread(token, ast) => {
                let span = ast.span().union(token.span);
                let expression = Expression::convert(analyzer, ast);
                Self {
                    span,
                    expression,
                    is_spread: true,
                }
            }
        }
    }

    pub(super) fn spread(expression: Expression) -> Self {
        Self {
            is_spread: true,
            ..Self::from(expression)
        }
    }
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

impl FromIterator<Element> for Pack {
    fn from_iter<T: IntoIterator<Item = Element>>(iter: T) -> Self {
        let values: Vec<_> = iter.into_iter().collect();
        Self { values }
    }
}

impl Extend<Element> for Pack {
    fn extend<T: IntoIterator<Item = Element>>(&mut self, iter: T) {
        self.values.extend(iter)
    }
}
