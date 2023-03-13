use super::*;

#[derive(Clone, Debug)]
pub enum Code {
    Modification(Box<Assignment>),
    Evaluation(Box<Evaluation>),
    Direction(Box<Direction>),
    Scope(Box<Scope>),
}

impl From<Assignment> for Code {
    fn from(assignment: Assignment) -> Self {
        Self::Modification(Box::new(assignment))
    }
}

impl From<Evaluation> for Code {
    fn from(evaluation: Evaluation) -> Self {
        Self::Evaluation(Box::new(evaluation))
    }
}

impl From<Direction> for Code {
    fn from(direction: Direction) -> Self {
        Self::Direction(Box::new(direction))
    }
}

impl From<Scope> for Code {
    fn from(scope: Scope) -> Self {
        Self::Scope(Box::new(scope))
    }
}
