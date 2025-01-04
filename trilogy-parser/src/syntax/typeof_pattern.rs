use super::{pattern::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct TypeofPattern {
    pub type_of: Token,
    pub pattern: Pattern,
    span: Span,
}

impl Spanned for TypeofPattern {
    fn span(&self) -> Span {
        self.span
    }
}

impl TypeofPattern {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let type_of = parser.expect(KwTypeof).unwrap();
        let pattern = Pattern::parse_precedence(parser, Precedence::Unary)?;
        Ok(Self {
            span: type_of.span.union(pattern.span()),
            type_of,
            pattern,
        })
    }
}

impl TryFrom<UnaryOperation> for TypeofPattern {
    type Error = SyntaxError;

    fn try_from(value: UnaryOperation) -> Result<Self, Self::Error> {
        let span = value.span();
        match value.operator {
            UnaryOperator::Typeof(token) => Ok(Self {
                span,
                type_of: token,
                pattern: value.operand.try_into()?,
            }),
            _ => Err(SyntaxError::new(
                value.span(),
                "incorrect operator for typeof pattern",
            )),
        }
    }
}
