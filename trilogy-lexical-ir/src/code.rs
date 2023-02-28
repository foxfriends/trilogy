use super::*;

#[derive(Clone, Debug)]
pub enum Code {
    // Basic statements
    Assignment(Box<Assignment>),
    Explicit(Box<Explicit>),
    Implicit(Box<Implicit>),
    // Three kinds of control flow:
    Loop(Vec<Branch>),   // `for` and `while`
    Branch(Vec<Branch>), // `if` and `match
    Handle(Vec<Branch>), // `when` and `given`
    // Then this guy, just there for practical reasons
    Scope(Box<Scope>),
}
