use super::*;

#[derive(Clone, Debug)]
pub enum Code {
    // Grouping
    Scope(Box<Scope>),

    // Primitive operations
    // TODO: how to do these...

    // Three kinds of control flow:
    Loop(Vec<Branch>),   // `for` and `while`
    Branch(Vec<Branch>), // `if` and `match
    Handle(Vec<Branch>), // `when` and `given`
}
