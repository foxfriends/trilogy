use super::{pattern::Precedence, *};
use crate::{Parser, Spanned};
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct NegativePattern {
    pub minus: Token,
    pub pattern: Pattern,
}

impl NegativePattern {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let minus = parser
            .expect(OpMinus)
            .expect("Caller should have found this");
        let pattern = Pattern::parse_precedence(parser, Precedence::Unary)?;
        Ok(Self { minus, pattern })
    }

    pub fn minus_token(&self) -> &Token {
        &self.minus
    }
}

impl TryFrom<UnaryOperation> for NegativePattern {
    type Error = SyntaxError;

    fn try_from(value: UnaryOperation) -> Result<Self, Self::Error> {
        match value.operator {
            UnaryOperator::Negate(token) => Ok(Self {
                minus: token,
                pattern: value.operand.try_into()?,
            }),
            _ => Err(SyntaxError::new(
                value.span(),
                "incorrect operator for negative pattern",
            )),
        }
    }
}
