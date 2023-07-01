use num::ToPrimitive;

use super::{Error, Execution, Program};
use crate::{runtime::Number, Instruction, Value};
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
            let instruction = ex.read_instruction(&self.program.instructions)?;
            match instruction {
                Instruction::Const => {
                    let value = ex.read_offset(&self.program.instructions)?;
                    ex.cactus
                        .push(self.program.constants[value as usize].clone());
                }
                Instruction::Load => {
                    let offset = ex.read_offset(&self.program.instructions)?;
                    ex.cactus.push(
                        ex.cactus
                            .at(offset as usize)
                            .cloned()
                            .ok_or(Error::InternalRuntimeError)?,
                    );
                }
                Instruction::Set => {
                    let offset = ex.read_offset(&self.program.instructions)?;
                    let value = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    *ex.cactus
                        .at_mut(offset as usize)
                        .ok_or(Error::InternalRuntimeError)? = value;
                }
                Instruction::Pop => {
                    ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                }
                Instruction::Add => {
                    let rhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    let lhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    match lhs + rhs {
                        Ok(val) => ex.cactus.push(val),
                        Err(..) => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::Subtract => {
                    let rhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    let lhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    match lhs - rhs {
                        Ok(val) => ex.cactus.push(val),
                        Err(..) => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::Multiply => {
                    let rhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    let lhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    match lhs * rhs {
                        Ok(val) => ex.cactus.push(val),
                        Err(..) => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::Divide => {
                    let rhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    let lhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    match lhs / rhs {
                        Ok(val) => ex.cactus.push(val),
                        Err(..) => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::Remainder => {
                    let rhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    let lhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    match lhs % rhs {
                        Ok(val) => ex.cactus.push(val),
                        Err(..) => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::IntDivide => {
                    let rhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    let lhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    match lhs / rhs {
                        Ok(Value::Number(val)) => {
                            ex.cactus
                                .push(Value::Number(Number::from(val.as_complex().re.floor())));
                        }
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::Power => {
                    let rhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    let lhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    match (lhs, rhs) {
                        (Value::Number(..), Value::Number(..)) => todo!("surprisingly hard"),
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::Negate => {
                    let val = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    match -val {
                        Ok(val) => ex.cactus.push(val),
                        Err(..) => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::Glue => {
                    let rhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    let lhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    match (lhs, rhs) {
                        (Value::String(lhs), Value::String(rhs)) => {
                            ex.cactus.push(Value::String(lhs + &rhs))
                        }
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::Access => {
                    let rhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    let lhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    match (lhs, rhs) {
                        (Value::Record(record), rhs) => match record.get(&rhs) {
                            Some(value) => ex.cactus.push(value),
                            None => todo!("yield 'MIA"),
                        },
                        (Value::String(lhs), Value::Number(rhs)) => {
                            let ch = rhs
                                .as_uinteger()
                                .and_then(|index| index.to_usize())
                                .and_then(|index| lhs.chars().nth(index));
                            match ch {
                                Some(ch) => ex.cactus.push(Value::Char(ch)),
                                None => todo!("yield 'MIA"),
                            }
                        }
                        (Value::Bits(lhs), Value::Number(rhs)) => {
                            let val = rhs
                                .as_uinteger()
                                .and_then(|index| index.to_usize())
                                .and_then(|index| lhs.get(index));
                            match val {
                                Some(val) => ex.cactus.push(Value::Bool(val)),
                                None => todo!("yield 'MIA"),
                            }
                        }
                        (Value::Array(lhs), Value::Number(rhs)) => {
                            let val = rhs
                                .as_uinteger()
                                .and_then(|index| index.to_usize())
                                .and_then(|index| lhs.get(index));
                            match val {
                                Some(val) => ex.cactus.push(val),
                                None => todo!("yield 'MIA"),
                            }
                        }
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::Assign => {
                    let value = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    let rhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    let lhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    match (lhs, rhs, value) {
                        (Value::Record(record), rhs, value) => {
                            record.insert(rhs, value);
                        }
                        (Value::Array(lhs), Value::Number(rhs), value) => {
                            let index = rhs.as_uinteger().and_then(|index| index.to_usize());
                            match index {
                                Some(index) => lhs.set(index, value),
                                None => todo!("yield 'MIA"),
                            }
                        }
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::Not => {
                    let val = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    match val {
                        Value::Bool(val) => ex.cactus.push(Value::Bool(!val)),
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::And => {
                    let rhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    let lhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    match (lhs, rhs) {
                        (Value::Bool(lhs), Value::Bool(rhs)) => {
                            ex.cactus.push(Value::Bool(lhs && rhs))
                        }
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::Or => {
                    let rhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    let lhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    match (lhs, rhs) {
                        (Value::Bool(lhs), Value::Bool(rhs)) => {
                            ex.cactus.push(Value::Bool(lhs || rhs))
                        }
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::BitwiseAnd => {
                    let lhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    let rhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    match lhs & rhs {
                        Ok(val) => ex.cactus.push(val),
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::BitwiseOr => {
                    let lhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    let rhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    match lhs | rhs {
                        Ok(val) => ex.cactus.push(val),
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::BitwiseXor => {
                    let lhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    let rhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    match lhs ^ rhs {
                        Ok(val) => ex.cactus.push(val),
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::BitwiseNeg => {
                    let val = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    match val {
                        Value::Bits(val) => ex.cactus.push(Value::Bits(!val)),
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::LeftShift => {
                    let lhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    let rhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    match lhs << rhs {
                        Ok(val) => ex.cactus.push(val),
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                Instruction::RightShift => {
                    let lhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    let rhs = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    match lhs >> rhs {
                        Ok(val) => ex.cactus.push(val),
                        _ => return Err(Error::RuntimeTypeError),
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
                    let right = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    let left = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    let mut branch = ex.branch();
                    ex.cactus.push(left);
                    branch.cactus.push(right);
                    self.executions.push_back(branch);
                }
                Instruction::Exit => {
                    // When run in embedded mode, the exit value can be any value. The
                    // interpreter binary can decide how to handle that exit value when
                    // passing off to the OS.
                    let value = ex.cactus.pop().ok_or(Error::InternalRuntimeError)?;
                    self.executions.clear();
                    return Ok(value);
                }
                _ => todo!(),
            }
        }
        Err(Error::ExecutionFizzledError)
    }
}
