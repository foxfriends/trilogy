use super::*;

#[derive(Clone, Debug)]
pub enum Code {
    Scope(Box<Scope>),
    Loop(Vec<Branch>),   // `for` and `while`
    Branch(Vec<Branch>), // `if` and `match
    Handle(Vec<Branch>), // `when` and `given`
}
