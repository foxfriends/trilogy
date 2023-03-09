use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Direction {
    span: Span,
    body: Step,
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
