use super::{pattern::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct NegativePattern {
    pub minus: Token,
    pub pattern: Pattern,
    span: Span,
}

impl NegativePattern {
    pub(crate) fn new(minus: Token, pattern: Pattern) -> Self {
        Self {
            span: minus.span.union(pattern.span()),
            minus,
            pattern,
        }
    }

    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let minus = parser
            .expect(OpMinus)
            .expect("Caller should have found this");
        let pattern = Pattern::parse_precedence(parser, Precedence::Unary)?;
        Ok(Self::new(minus, pattern))
    }
}

impl Spanned for NegativePattern {
    fn span(&self) -> Span {
        self.span
    }
}

impl TryFrom<UnaryOperation> for NegativePattern {
    type Error = SyntaxError;

    fn try_from(value: UnaryOperation) -> Result<Self, Self::Error> {
        let span = value.span();
        match value.operator {
            UnaryOperator::Negate(minus) => Ok(Self {
                span,
                minus,
                pattern: value.operand.try_into()?,
            }),
            _ => Err(SyntaxError::new(
                value.span(),
                "incorrect operator for negative pattern",
            )),
        }
    }
}
