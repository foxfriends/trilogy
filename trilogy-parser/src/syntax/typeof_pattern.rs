use super::{pattern::Precedence, *};
use crate::{Parser, Spanned};
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct TypeofPattern {
    pub type_of: Token,
    pub pattern: Pattern,
}

impl TypeofPattern {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let type_of = parser
            .expect(KwTypeof)
            .expect("Caller should have found this");
        let pattern = Pattern::parse_precedence(parser, Precedence::Unary)?;
        Ok(Self { type_of, pattern })
    }
}

impl TryFrom<UnaryOperation> for TypeofPattern {
    type Error = SyntaxError;

    fn try_from(value: UnaryOperation) -> Result<Self, Self::Error> {
        match value.operator {
            UnaryOperator::Typeof(token) => Ok(Self {
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
