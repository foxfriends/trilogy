use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Direction {
    pub span: Span,
    pub body: Step,
}

#[derive(Clone, Debug)]
pub enum Step {
    Conjunction(Box<Direction>),
    Disjunction(Box<Direction>),
    Implication(Box<Direction>),
    Selection(Box<Direction>),
    Negation(Box<Direction>),
    Unification(Box<BinaryOperation>),
    Iteration(Box<BinaryOperation>),
    Invocation(Box<Call>),
    Evaluation(Box<Evaluation>),
    Confirmation,
    Contradiction,
}
