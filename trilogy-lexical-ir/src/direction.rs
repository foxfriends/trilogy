use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Direction {
    pub span: Span,
    pub body: Step,
}

#[derive(Clone, Debug)]
pub enum Step {
    Conjunction(Box<BinaryDirection>),
    Disjunction(Box<BinaryDirection>),
    Implication(Box<BinaryDirection>),
    Selection(Box<BinaryDirection>),
    Negation(Box<BinaryDirection>),
    Unification(Box<BinaryOperation>),
    Iteration(Box<BinaryOperation>),
    Invocation(Box<Call>),
    Evaluation(Box<Evaluation>),
    Confirmation,
    Contradiction,
    Violation(Violation),
}

#[derive(Clone, Debug)]
pub struct BinaryDirection {
    pub lhs: Direction,
    pub rhs: Direction,
}

impl BinaryDirection {
    pub fn new(lhs: Direction, rhs: Direction) -> Self {
        Self { lhs, rhs }
    }
}
