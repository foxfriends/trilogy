use super::*;

#[derive(Clone, Debug)]
pub enum Code {
    Modification(Box<Assignment>),
    Evaluation(Box<Evaluation>),
    Direction(Vec<Direction>),
    Scope(Box<Scope>),
}
