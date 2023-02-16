use super::*;
use crate::Parser;
use trilogy_scanner::{
    Token,
    TokenType::{self, *},
};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct AssignmentStatement {
    pub lhs: Expression,
    pub strategy: AssignmentStrategy,
    pub rhs: Expression,
}

impl AssignmentStatement {
    pub(crate) const ASSIGNMENT_OPERATOR: [TokenType; 20] = [
        OpEq,
        OpAmpAmpEq,
        OpPipePipeEq,
        OpAmpEq,
        OpPipeEq,
        OpCaretEq,
        OpShrEq,
        OpShlEq,
        OpGlueEq,
        OpPlusEq,
        OpMinusEq,
        OpStarEq,
        OpSlashEq,
        OpSlashSlashEq,
        OpPercentEq,
        OpStarStarEq,
        OpLtLtEq,
        OpGtGtEq,
        OpColonEq,
        OpDotEq,
    ];

    pub(crate) fn parse(parser: &mut Parser, lhs: Expression) -> SyntaxResult<Self> {
        let strategy = AssignmentStrategy::parse(parser)?;
        let rhs = Expression::parse(parser)?;
        Ok(Self { lhs, strategy, rhs })
    }
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum AssignmentStrategy {
    Direct(Token),
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
    Access(Token),
    Cons(Token),
}

impl AssignmentStrategy {
    fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(AssignmentStatement::ASSIGNMENT_OPERATOR)
            .map_err(|token| {
                parser.expected(token, "expected assignment operator (ending with `=`)")
            })?;
        Ok(match token.token_type {
            OpEq => Self::Direct(token),
            OpAmpAmpEq => Self::And(token),
            OpPipePipeEq => Self::Or(token),
            OpAmpEq => Self::BitwiseAnd(token),
            OpPipeEq => Self::BitwiseOr(token),
            OpCaretEq => Self::BitwiseXor(token),
            OpShrEq => Self::LeftShift(token),
            OpShlEq => Self::RightShift(token),
            OpGlueEq => Self::Glue(token),
            OpPlusEq => Self::Add(token),
            OpMinusEq => Self::Subtract(token),
            OpStarEq => Self::Multiply(token),
            OpSlashEq => Self::Divide(token),
            OpSlashSlashEq => Self::IntDivide(token),
            OpPercentEq => Self::Remainder(token),
            OpStarStarEq => Self::Power(token),
            OpLtLtEq => Self::RCompose(token),
            OpGtGtEq => Self::Compose(token),
            OpColonEq => Self::Cons(token),
            OpDotEq => Self::Access(token),
            _ => unreachable!(),
        })
    }
}
