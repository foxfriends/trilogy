use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct BinaryOperation {
    pub lhs: Expression,
    pub operator: BinaryOperator,
    pub rhs: Expression,
}

impl BinaryOperation {
    pub(crate) fn new(
        lhs: impl Into<Expression>,
        operator: BinaryOperator,
        rhs: impl Into<Expression>,
    ) -> Self {
        Self {
            lhs: lhs.into(),
            operator,
            rhs: rhs.into(),
        }
    }
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum BinaryOperator {
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
    ReferenceEquality(Token),
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
