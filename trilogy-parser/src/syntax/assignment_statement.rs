use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct AssignmentStatement {
    pub lhs: Expression,
    pub strategy: AssignmentStrategy,
    pub rhs: Expression,
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum AssignmentStrategy {
    Direct(Token),
    Function(Identifier),
    And(Token),
    Or(Token),
    Add(Token),
    Subtract(Token),
    Multiply(Token),
    Divide(Token),
    Remainder(Token),
    Power(Token),
    IntDivide(Token),
    BitwiseAnd(Token),
    BitwiseOr(Token),
    BitwiseXor(Token),
    LeftShift(Token),
    RightShift(Token),
    Glue(Token),
    Compose(Token),
    RCompose(Token),
}
