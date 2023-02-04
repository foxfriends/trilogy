use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct AssignmentStatement {
    pub lhs: LValue,
    pub strategy: AssignmentStrategy,
    pub rhs: Expression,
}

#[derive(Clone, Debug)]
pub enum LValue {
    Pattern(Box<Pattern>),
    Member(Box<MemberAccess>),
}

#[derive(Clone, Debug)]
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
