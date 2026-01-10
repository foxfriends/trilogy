use super::{expression::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug)]
pub struct UnaryOperation {
    pub operator: UnaryOperator,
    pub operand: Expression,
    pub span: Span,
}

impl Spanned for UnaryOperation {
    fn span(&self) -> Span {
        self.span
    }
}

impl UnaryOperation {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Result<Self, Pattern>> {
        let operator = UnaryOperator::parse(parser);
        let operand = Expression::parse_or_pattern_precedence(parser, operator.precedence())?;
        match operand {
            Ok(operand) => Ok(Ok(Self {
                span: operator.span().union(operand.span()),
                operator,
                operand,
            })),
            Err(pattern) => match operator {
                UnaryOperator::Negate(token) => Ok(Err(Pattern::Negative(Box::new(
                    NegativePattern::new(token, pattern),
                )))),
                _ => Err(SyntaxError::new(
                    pattern.span(),
                    "expected an expression for the unary operation, but found a pattern",
                )),
            },
        }
    }
}

#[derive(Clone, Debug, Spanned)]
pub enum UnaryOperator {
    Negate(Token),
    Not(Token),
    Invert(Token),
    Yield(Token),
    Typeof(Token),
}

impl UnaryOperator {
    fn parse(parser: &mut Parser) -> Self {
        let token = parser
            .expect([OpBang, OpMinus, OpTilde, KwYield, KwTypeof])
            .expect("Caller should have found one of these");
        match token.token_type {
            OpBang => Self::Not(token),
            OpMinus => Self::Negate(token),
            OpTilde => Self::Invert(token),
            KwYield => Self::Yield(token),
            KwTypeof => Self::Typeof(token),
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
