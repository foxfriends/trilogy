use super::{expression::Precedence, *};
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct BinaryOperation {
    pub lhs: Expression,
    pub operator: BinaryOperator,
    pub rhs: Expression,
}

impl BinaryOperation {
    pub(crate) fn parse(parser: &mut Parser, lhs: impl Into<Expression>) -> SyntaxResult<Self> {
        let operator = BinaryOperator::parse(parser);
        let rhs = Expression::parse_precedence(parser, operator.precedence())?;
        Ok(BinaryOperation {
            lhs: lhs.into(),
            operator,
            rhs,
        })
    }
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum BinaryOperator {
    Access(Token),
    And(Token),
    Or(Token),
    Add(Token),
    Subtract(Token),
    Multiply(Token),
    Divide(Token),
    Remainder(Token),
    Power(Token),
    IntDivide(Token),
    StructuralEquality(Token),
    StructuralInequality(Token),
    ReferenceEquality(Token),
    ReferenceInequality(Token),
    Lt(Token),
    Gt(Token),
    Leq(Token),
    Geq(Token),
    BitwiseAnd(Token),
    BitwiseOr(Token),
    BitwiseXor(Token),
    LeftShift(Token),
    RightShift(Token),
    Sequence(Token),
    Cons(Token),
    Glue(Token),
    Compose(Token),
    RCompose(Token),
    Pipe(Token),
    RPipe(Token),
}

impl BinaryOperator {
    fn parse(parser: &mut Parser) -> Self {
        let token = parser.consume();
        match token.token_type {
            OpDot => Self::Access(token),
            OpAmpAmp => Self::And(token),
            OpPipePipe => Self::Or(token),
            OpPlus => Self::Add(token),
            OpMinus => Self::Subtract(token),
            OpStar => Self::Multiply(token),
            OpSlash => Self::Divide(token),
            OpSlashSlash => Self::IntDivide(token),
            OpPercent => Self::Remainder(token),
            OpStarStar => Self::Power(token),
            OpEqEq => Self::StructuralEquality(token),
            OpBangEq => Self::StructuralInequality(token),
            OpEqEqEq => Self::ReferenceEquality(token),
            OpBangEqEq => Self::ReferenceInequality(token),
            OpLt => Self::Lt(token),
            OpGt => Self::Gt(token),
            OpLtEq => Self::Leq(token),
            OpGtEq => Self::Geq(token),
            OpAmp => Self::BitwiseAnd(token),
            OpPipe => Self::BitwiseOr(token),
            OpCaret => Self::BitwiseXor(token),
            OpShl => Self::LeftShift(token),
            OpShr => Self::RightShift(token),
            OpComma => Self::Sequence(token),
            OpColon => Self::Cons(token),
            OpGlue => Self::Glue(token),
            OpGtGt => Self::Compose(token),
            OpLtLt => Self::RCompose(token),
            OpPipeGt => Self::Pipe(token),
            OpLtPipe => Self::RPipe(token),
            _ => unreachable!(),
        }
    }

    fn precedence(&self) -> Precedence {
        match self {
            BinaryOperator::Access(..) => Precedence::Access,
            BinaryOperator::And(..) => Precedence::And,
            BinaryOperator::Or(..) => Precedence::Or,
            BinaryOperator::Add(..) | BinaryOperator::Subtract(..) => Precedence::Term,
            BinaryOperator::Multiply(..)
            | BinaryOperator::Divide(..)
            | BinaryOperator::IntDivide(..)
            | BinaryOperator::Remainder(..) => Precedence::Factor,
            BinaryOperator::Power(..) => Precedence::Exponent,
            BinaryOperator::StructuralEquality(..)
            | BinaryOperator::ReferenceEquality(..)
            | BinaryOperator::StructuralInequality(..)
            | BinaryOperator::ReferenceInequality(..) => Precedence::Equality,
            BinaryOperator::Lt(..)
            | BinaryOperator::Gt(..)
            | BinaryOperator::Geq(..)
            | BinaryOperator::Leq(..) => Precedence::Comparison,
            BinaryOperator::BitwiseAnd(..) => Precedence::BitwiseAnd,
            BinaryOperator::BitwiseOr(..) => Precedence::BitwiseOr,
            BinaryOperator::BitwiseXor(..) => Precedence::BitwiseXor,
            BinaryOperator::LeftShift(..) | BinaryOperator::RightShift(..) => {
                Precedence::BitwiseShift
            }
            BinaryOperator::Sequence(..) => Precedence::Sequence,
            BinaryOperator::Cons(..) => Precedence::Cons,
            BinaryOperator::Glue(..) => Precedence::Glue,
            BinaryOperator::Compose(..) => Precedence::Compose,
            BinaryOperator::RCompose(..) => Precedence::RCompose,
            BinaryOperator::Pipe(..) => Precedence::Pipe,
            BinaryOperator::RPipe(..) => Precedence::RPipe,
        }
    }
}
