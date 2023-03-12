use super::*;

#[derive(Clone, Debug)]
pub enum Code {
    Modification(Box<Assignment>),
    Evaluation(Box<Evaluation>),
    Direction(Box<Direction>),
    Scope(Box<Scope>),
}
