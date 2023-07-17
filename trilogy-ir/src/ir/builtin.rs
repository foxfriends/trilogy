use super::*;
use trilogy_parser::{syntax, Spanned};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Builtin {
    Negate,
    Not,
    Invert,
    Access,
    ModuleAccess,
    And,
    Or,
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Power,
    IntDivide,
    StructuralEquality,
    StructuralInequality,
    ReferenceEquality,
    ReferenceInequality,
    Lt,
    Gt,
    Leq,
    Geq,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,
    Sequence,
    Cons,
    Glue,
    Compose,
    RCompose,
    Pipe,
    RPipe,
    Construct,
    Array,
    Set,
    Record,
    Is,
    Pin,
    For,
    Yield,
    Resume,
    Cancel,
    Return,
    Break,
    Continue,
    Exit,
}

impl Builtin {
    pub(super) fn convert(ast: syntax::KeywordReference) -> Expression {
        let span = ast.span();
        let op = match ast.keyword {
            syntax::Keyword::And(..) => Self::And,
            syntax::Keyword::Or(..) => Self::Or,
            syntax::Keyword::Add(..) => Self::Add,
            syntax::Keyword::Subtract(..) => Self::Subtract,
            syntax::Keyword::Multiply(..) => Self::Multiply,
            syntax::Keyword::Divide(..) => Self::Divide,
            syntax::Keyword::Remainder(..) => Self::Remainder,
            syntax::Keyword::Power(..) => Self::Power,
            syntax::Keyword::IntDivide(..) => Self::IntDivide,
            syntax::Keyword::StructuralEquality(..) => Self::StructuralEquality,
            syntax::Keyword::ReferenceEquality(..) => Self::ReferenceEquality,
            syntax::Keyword::StructuralInequality(..) => Self::StructuralInequality,
            syntax::Keyword::ReferenceInequality(..) => Self::ReferenceInequality,
            syntax::Keyword::Lt(..) => Self::Lt,
            syntax::Keyword::Gt(..) => Self::Gt,
            syntax::Keyword::Leq(..) => Self::Leq,
            syntax::Keyword::Geq(..) => Self::Geq,
            syntax::Keyword::BitwiseAnd(..) => Self::BitwiseAnd,
            syntax::Keyword::BitwiseOr(..) => Self::BitwiseOr,
            syntax::Keyword::BitwiseXor(..) => Self::BitwiseXor,
            syntax::Keyword::LeftShift(..) => Self::LeftShift,
            syntax::Keyword::RightShift(..) => Self::RightShift,
            syntax::Keyword::Sequence(..) => Self::Sequence,
            syntax::Keyword::Cons(..) => Self::Cons,
            syntax::Keyword::Glue(..) => Self::Glue,
            syntax::Keyword::Compose(..) => Self::Compose,
            syntax::Keyword::RCompose(..) => Self::RCompose,
            syntax::Keyword::Pipe(..) => Self::Pipe,
            syntax::Keyword::RPipe(..) => Self::RPipe,
            syntax::Keyword::Not(..) => Self::Not,
            syntax::Keyword::Invert(..) => Self::Invert,
            syntax::Keyword::Yield(..) => Self::Yield,
            syntax::Keyword::Resume(..) => Self::Resume,
            syntax::Keyword::Cancel(..) => Self::Cancel,
            syntax::Keyword::Return(..) => Self::Return,
            syntax::Keyword::Break(..) => Self::Break,
            syntax::Keyword::Continue(..) => Self::Continue,
        };
        Expression::builtin(span, op)
    }

    pub(super) fn convert_binary(ast: syntax::BinaryOperator) -> Expression {
        let span = ast.span();
        let op = match ast {
            syntax::BinaryOperator::Access(..) => Self::Access,
            syntax::BinaryOperator::And(..) => Self::And,
            syntax::BinaryOperator::Or(..) => Self::Or,
            syntax::BinaryOperator::Add(..) => Self::Add,
            syntax::BinaryOperator::Subtract(..) => Self::Subtract,
            syntax::BinaryOperator::Multiply(..) => Self::Multiply,
            syntax::BinaryOperator::Divide(..) => Self::Divide,
            syntax::BinaryOperator::Remainder(..) => Self::Remainder,
            syntax::BinaryOperator::Power(..) => Self::Power,
            syntax::BinaryOperator::IntDivide(..) => Self::IntDivide,
            syntax::BinaryOperator::StructuralEquality(..) => Self::StructuralEquality,
            syntax::BinaryOperator::StructuralInequality(..) => Self::StructuralInequality,
            syntax::BinaryOperator::ReferenceEquality(..) => Self::ReferenceEquality,
            syntax::BinaryOperator::ReferenceInequality(..) => Self::ReferenceInequality,
            syntax::BinaryOperator::Lt(..) => Self::Lt,
            syntax::BinaryOperator::Gt(..) => Self::Gt,
            syntax::BinaryOperator::Leq(..) => Self::Leq,
            syntax::BinaryOperator::Geq(..) => Self::Geq,
            syntax::BinaryOperator::BitwiseAnd(..) => Self::BitwiseAnd,
            syntax::BinaryOperator::BitwiseOr(..) => Self::BitwiseOr,
            syntax::BinaryOperator::BitwiseXor(..) => Self::BitwiseXor,
            syntax::BinaryOperator::LeftShift(..) => Self::LeftShift,
            syntax::BinaryOperator::RightShift(..) => Self::RightShift,
            syntax::BinaryOperator::Sequence(..) => Self::Sequence,
            syntax::BinaryOperator::Cons(..) => Self::Cons,
            syntax::BinaryOperator::Glue(..) => Self::Glue,
            syntax::BinaryOperator::Compose(..) => Self::Compose,
            syntax::BinaryOperator::RCompose(..) => Self::RCompose,
            syntax::BinaryOperator::Pipe(..) => Self::Pipe,
            syntax::BinaryOperator::RPipe(..) => Self::RPipe,
        };
        Expression::builtin(span, op)
    }

    pub(super) fn convert_unary(ast: syntax::UnaryOperator) -> Expression {
        let span = ast.span();
        let op = match ast {
            syntax::UnaryOperator::Invert(..) => Builtin::Invert,
            syntax::UnaryOperator::Negate(..) => Builtin::Negate,
            syntax::UnaryOperator::Not(..) => Builtin::Not,
            syntax::UnaryOperator::Yield(..) => Builtin::Yield,
        };
        Expression::builtin(span, op)
    }
}
