use super::*;

#[derive(Clone, Debug)]
pub enum Code {
    Assignment(Box<Assignment>),
    Explicit(Box<Explicit>),
    Implicit(Box<Implicit>),
    Evaluation(Box<Evaluation>),
    Loop(Vec<Branch>),
    Branch(Vec<Branch>),
    Handle(Vec<Branch>),
    Scope(Box<Scope>),
}
