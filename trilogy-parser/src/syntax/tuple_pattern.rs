use super::{pattern::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct TuplePattern {
    pub lhs: Pattern,
    pub cons: Token,
    pub rhs: Pattern,
    span: Span,
}

impl Spanned for TuplePattern {
    fn span(&self) -> Span {
        self.span
    }
}

impl TuplePattern {
    pub(crate) fn new(lhs: Pattern, cons: Token, rhs: Pattern) -> Self {
        Self {
            span: lhs.span().union(rhs.span()),
            lhs,
            cons,
            rhs,
        }
    }

    pub(crate) fn parse(parser: &mut Parser, lhs: Pattern) -> SyntaxResult<Self> {
        let cons = parser
            .expect(OpColon)
            .expect("Caller should have found this");
        let rhs = Pattern::parse_precedence(parser, Precedence::Cons)?;
        Ok(Self {
            span: lhs.span().union(rhs.span()),
            lhs,
            cons,
            rhs,
        })
    }
}

impl TryFrom<BinaryOperation> for TuplePattern {
    type Error = SyntaxError;

    fn try_from(value: BinaryOperation) -> Result<Self, Self::Error> {
        let span = value.span();
        match value.operator {
            BinaryOperator::Cons(token) => Ok(Self {
                span,
                lhs: value.lhs.try_into()?,
                cons: token,
                rhs: value.rhs.try_into()?,
            }),
            _ => Err(SyntaxError::new(
                value.span(),
                "incorrect operator for tuple pattern",
            )),
        }
    }
}
