use super::{expression::Precedence, *};
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct UnaryOperation {
    pub operator: UnaryOperator,
    pub operand: Expression,
}

impl UnaryOperation {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let operator = UnaryOperator::parse(parser);
        let operand = Expression::parse_precedence(parser, operator.precedence())?;
        Ok(Self { operator, operand })
    }
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum UnaryOperator {
    Negate(Token),
    Not(Token),
    Invert(Token),
    Yield(Token),
}

impl UnaryOperator {
    fn parse(parser: &mut Parser) -> Self {
        let token = parser
            .expect([OpBang, OpMinus, OpTilde, KwYield])
            .expect("Caller should have found one of these");
        match token.token_type {
            OpBang => Self::Not(token),
            OpMinus => Self::Negate(token),
            OpTilde => Self::Invert(token),
            KwYield => Self::Yield(token),
            _ => unreachable!(),
        }
    }

    fn precedence(&self) -> Precedence {
        if matches!(self, Self::Yield(..)) {
            Precedence::Continuation
        } else {
            Precedence::Unary
        }
    }
}
