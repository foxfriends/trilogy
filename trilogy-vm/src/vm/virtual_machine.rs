use super::{Error, Execution, Program};
use crate::{Instruction, Value};
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct VirtualMachine {
    program: Program,
    executions: VecDeque<Execution>,
}

impl VirtualMachine {
    pub fn load(program: Program) -> Self {
        Self {
            program,
            executions: VecDeque::with_capacity(8),
        }
    }

    pub fn run(&mut self) -> Result<Value, Error> {
        self.executions.push_back(Execution::default());
        // In future, multiple executions will likely be run in parallel on different
        // threads. Maybe this should be a compile time option, where the alternatives
        // are depth-first, or breadth-first multitasking.
        //
        // For now, the only option is depth-first multitasking, as it is easiest to
        // reason about and implement, as well as most likely to be performant in most
        // simple situations, which is all that we will have to deal with while this
        // VM remains a (relative) toy.
        let ep = 0;
        while !self.executions.is_empty() {
            let ex = &mut self.executions[ep];
            let ip = ex.ip;
            ex.ip += 1;
            let instruction = self.program.instructions[ip];
            match instruction {
                Instruction::Add => {
                    let rhs = ex.cactus.pop().unwrap();
                    let lhs = ex.cactus.pop().unwrap();
                    match lhs + rhs {
                        Ok(val) => ex.cactus.push(val),
                        Err(..) => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::Subtract => {
                    let rhs = ex.cactus.pop().unwrap();
                    let lhs = ex.cactus.pop().unwrap();
                    match lhs - rhs {
                        Ok(val) => ex.cactus.push(val),
                        Err(..) => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::Multiply => {
                    let rhs = ex.cactus.pop().unwrap();
                    let lhs = ex.cactus.pop().unwrap();
                    match lhs * rhs {
                        Ok(val) => ex.cactus.push(val),
                        Err(..) => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::Divide => {
                    let rhs = ex.cactus.pop().unwrap();
                    let lhs = ex.cactus.pop().unwrap();
                    match lhs / rhs {
                        Ok(val) => ex.cactus.push(val),
                        Err(..) => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::Remainder => {
                    let rhs = ex.cactus.pop().unwrap();
                    let lhs = ex.cactus.pop().unwrap();
                    match lhs % rhs {
                        Ok(val) => ex.cactus.push(val),
                        Err(..) => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::Fizzle => {
                    // This just ends EVERYTHING
                    //
                    // Is there any cleanup that has to be done? Or does Rust's
                    // RAII cause that cleanup to happen automatically?
                    //
                    // I suspect it is automatic.
                    self.executions.pop_front();
                }
                Instruction::Branch => {
                    // A branch requires two values on the stack; the two branches get the
                    // different values, respectively.
                    let right = ex.cactus.pop().unwrap();
                    let left = ex.cactus.pop().unwrap();
                    let mut branch = ex.branch();
                    ex.cactus.push(left);
                    branch.cactus.push(right);
                    self.executions.push_back(branch);
                }
                Instruction::Exit => {
                    // When run in embedded mode, the exit value can be any value. The
                    // interpreter binary can decide how to handle that exit value when
                    // passing off to the OS.
                    let value = ex.cactus.pop().unwrap();
                    self.executions.clear();
                    return Ok(value);
                }
                _ => todo!(),
            }
        }
        Err(Error::ExecutionFizzledError)
    }
}
