use super::{expression::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

/// A binary operation expression.
///
/// ```trilogy
/// lhs + rhs
/// ```
#[derive(Clone, Debug)]
pub struct BinaryOperation {
    pub lhs: Expression,
    pub operator: BinaryOperator,
    pub rhs: Expression,
    pub span: Span,
}

impl Spanned for BinaryOperation {
    fn span(&self) -> Span {
        self.span
    }
}

impl BinaryOperation {
    pub(crate) fn parse(
        parser: &mut Parser,
        lhs: Expression,
    ) -> SyntaxResult<Result<Self, Pattern>> {
        let operator = BinaryOperator::parse(parser);
        let rhs = Expression::parse_or_pattern_precedence(parser, operator.precedence())?;
        match rhs {
            Ok(rhs) => Ok(Ok(BinaryOperation {
                span: lhs.span().union(rhs.span()),
                lhs,
                operator,
                rhs,
            })),
            Err(rhs) => match operator {
                BinaryOperator::Glue(glue) => Ok(Err(Pattern::Glue(Box::new(GluePattern::new(
                    lhs.try_into().inspect_err(|err: &SyntaxError| {
                        parser.error(err.clone());
                    })?,
                    glue,
                    rhs,
                ))))),
                BinaryOperator::Cons(token) => {
                    Ok(Err(Pattern::Tuple(Box::new(TuplePattern::new(
                        lhs.try_into().inspect_err(|err: &SyntaxError| {
                            parser.error(err.clone());
                        })?,
                        token,
                        rhs,
                    )))))
                }
                _ => {
                    let err =
                        SyntaxError::new(rhs.span(), "expected an expression, but found a pattern");
                    parser.error(err.clone());
                    Err(err)
                }
            },
        }
    }
}

/// A binary operator, represented by a single token.
#[derive(Clone, Debug, Spanned)]
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
    LeftShiftExtend(Token),
    RightShiftExtend(Token),
    LeftShiftContract(Token),
    RightShiftContract(Token),
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
            OpShlEx => Self::LeftShiftExtend(token),
            OpShrEx => Self::RightShiftExtend(token),
            OpShlCon => Self::LeftShiftContract(token),
            OpShrCon => Self::RightShiftContract(token),
            OpColon => Self::Cons(token),
            OpGlue => Self::Glue(token),
            OpLtLt => Self::Compose(token),
            OpGtGt => Self::RCompose(token),
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
            BinaryOperator::LeftShift(..)
            | BinaryOperator::RightShift(..)
            | BinaryOperator::LeftShiftExtend(..)
            | BinaryOperator::RightShiftExtend(..)
            | BinaryOperator::LeftShiftContract(..)
            | BinaryOperator::RightShiftContract(..) => Precedence::BitwiseShift,
            BinaryOperator::Cons(..) => Precedence::Cons,
            BinaryOperator::Glue(..) => Precedence::Glue,
            BinaryOperator::Compose(..) => Precedence::Compose,
            BinaryOperator::RCompose(..) => Precedence::RCompose,
            BinaryOperator::Pipe(..) => Precedence::Pipe,
            BinaryOperator::RPipe(..) => Precedence::RPipe,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(binop_access: "a . b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::Access(_), .. }));
    test_parse!(binop_and: "a && b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::And(_), .. }));
    test_parse!(binop_or: "a || b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::Or(_), .. }));
    test_parse!(binop_plus: "a + b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::Add(_), .. }));
    test_parse!(binop_minus: "a - b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::Subtract(_), .. }));
    test_parse!(binop_multiply: "a * b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::Multiply(_), .. }));
    test_parse!(binop_divide: "a / b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::Divide(_), .. }));
    test_parse!(binop_int_divide: "a // b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::IntDivide(_), .. }));
    test_parse!(binop_remainder: "a % b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::Remainder(_), .. }));
    test_parse!(binop_power: "a ** b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::Power(_), .. }));
    test_parse!(binop_equal: "a == b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::StructuralEquality(_), .. }));
    test_parse!(binop_ref_equal: "a === b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::ReferenceEquality(_), .. }));
    test_parse!(binop_not_equal: "a != b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::StructuralInequality(_), .. }));
    test_parse!(binop_not_ref_equal: "a !== b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::ReferenceInequality(_), .. }));
    test_parse!(binop_lt: "a < b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::Lt(_), .. }));
    test_parse!(binop_gt: "a > b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::Gt(_), .. }));
    test_parse!(binop_leq: "a <= b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::Leq(_), .. }));
    test_parse!(binop_geq: "a >= b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::Geq(_), .. }));
    test_parse!(binop_bitwise_and: "a & b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::BitwiseAnd(_), .. }));
    test_parse!(binop_bitwise_or: "a | b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::BitwiseOr(_), .. }));
    test_parse!(binop_bitwise_xor: "a ^ b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::BitwiseXor(_), .. }));
    test_parse!(binop_bitwise_shl: "a <~ b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::LeftShift(_), .. }));
    test_parse!(binop_bitwise_shr: "a ~> b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::RightShift(_), .. }));
    test_parse!(binop_bitwise_shl_ex: "a <~~ b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::LeftShiftExtend(_), .. }));
    test_parse!(binop_bitwise_shr_ex: "a ~~> b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::RightShiftExtend(_), .. }));
    test_parse!(binop_bitwise_shl_con: "a <<~ b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::LeftShiftContract(_), .. }));
    test_parse!(binop_bitwise_shr_con: "a ~>> b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::RightShiftContract(_), .. }));
    test_parse!(binop_cons: "a : b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::Cons(_), .. }));
    test_parse!(binop_glue: "a <> b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::Glue(_), .. }));
    test_parse!(binop_compose: "a << b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::Compose(_), .. }));
    test_parse!(binop_rcompose: "a >> b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::RCompose(_), .. }));
    test_parse!(binop_pipe: "a |> b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::Pipe(_), .. }));
    test_parse!(binop_rpipe: "a <| b" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::RPipe(_), .. }));

    test_parse!(binop_with_unary: "a + -5" => Expression::parse => Expression::Binary(BinaryOperation { operator: BinaryOperator::Add(_), rhs: Expression::Unary(_), .. }));

    test_parse_error!(binop_not_and_operator: "a and b" => Expression::parse);
    test_parse_error!(binop_not_or_operator: "a or b" => Expression::parse);
    test_parse_error!(binop_not_seq_operator: "a , b" => Expression::parse);
}
