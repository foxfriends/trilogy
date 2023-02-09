use super::{value_expression::Precedence, *};
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
        let operand = ValueExpression::parse_precedence(parser, operator.precedence())?;
        Ok(Self {
            operator,
            operand: operand.into(),
        })
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
            .expect([KwNot, OpMinus, OpTilde, KwYield])
            .expect("Caller should have found one of these");
        match token.token_type {
            KwNot => Self::Not(token),
            OpMinus => Self::Negate(token),
            OpTilde => Self::Invert(token),
            KwYield => Self::Yield(token),
            _ => unreachable!(),
        }
    }

    fn precedence(&self) -> Precedence {
        if matches!(self, Self::Yield(..)) {
            Precedence::Yield
        } else {
            Precedence::Unary
        }
    }
}
