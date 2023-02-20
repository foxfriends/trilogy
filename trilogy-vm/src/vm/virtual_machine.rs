use super::{Continuation, Program};

#[derive(Clone, Debug)]
pub struct VirtualMachine {
    program: Program,
    continuations: Vec<Continuation>,
}

impl VirtualMachine {
    pub fn load(program: Program) -> Self {
        Self {
            program,
            continuations: vec![Continuation::default()],
        }
    }
}
