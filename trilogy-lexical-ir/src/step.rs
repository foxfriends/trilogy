use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub enum Step {
    Conjunction(Box<BinaryDirection>),
    Disjunction(Box<BinaryDirection>),
    Implication(Box<BinaryDirection>),
    Selection(Box<BinaryDirection>),
    Negation(Box<Direction>),
    Unification(Box<BinaryOperation>),
    Iteration(Box<BinaryOperation>),
    Invocation(Box<Call>),
    Evaluation(Box<Evaluation>),
    Confirmation,
    Contradiction,
    Violation(Box<Violation>),
}

macro_rules! binary {
    ($name:ident, $variant:ident) => {
        pub fn $name(lhs: Direction, rhs: Direction) -> Self {
            Self::$variant(Box::new(BinaryDirection::new(lhs, rhs)))
        }
    };
}

macro_rules! unary {
    ($name:ident, $variant:ident, $t:ty) => {
        pub fn $name(value: $t) -> Self {
            Self::$variant(Box::new(value))
        }
    };
}

impl Step {
    binary!(conjunction, Conjunction);
    binary!(disjunction, Disjunction);
    binary!(implication, Implication);
    binary!(selection, Selection);

    unary!(negation, Negation, Direction);
    unary!(invocation, Invocation, Call);
    unary!(evaluation, Evaluation, Evaluation);
    unary!(violation, Violation, Violation);

    pub fn unification(lhs: Evaluation, rhs: Evaluation) -> Self {
        Self::Unification(Box::new(BinaryOperation::new(lhs, rhs)))
    }

    pub fn iteration(lhs: Evaluation, rhs: Evaluation) -> Self {
        Self::Iteration(Box::new(BinaryOperation::new(lhs, rhs)))
    }

    pub fn at(self, span: Span) -> Direction {
        Direction { span, body: self }
    }
}
