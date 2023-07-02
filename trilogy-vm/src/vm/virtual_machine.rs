use super::{Error, Execution, Program};
use crate::bytecode::OpCode;
use crate::runtime::Number;
use crate::{ReferentialEq, StructuralEq};
use crate::{Tuple, Value};
use num::ToPrimitive;
use std::cmp::Ordering;
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
            let instruction = ex.read_opcode(&self.program.instructions)?;
            match instruction {
                OpCode::Const => {
                    let value = ex.read_offset(&self.program.instructions)?;
                    ex.stack.push(self.program.constants[value].clone());
                }
                OpCode::Load => {
                    let offset = ex.read_offset(&self.program.instructions)?;
                    ex.stack.push(ex.stack.at(offset)?);
                }
                OpCode::Set => {
                    let offset = ex.read_offset(&self.program.instructions)?;
                    let value = ex.stack.pop()?;
                    ex.stack.replace_at(offset, value)?;
                }
                OpCode::Pop => {
                    ex.stack.pop()?;
                }
                OpCode::Add => {
                    let rhs = ex.stack.pop()?;
                    let lhs = ex.stack.pop()?;
                    match lhs + rhs {
                        Ok(val) => ex.stack.push(val),
                        Err(..) => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::Subtract => {
                    let rhs = ex.stack.pop()?;
                    let lhs = ex.stack.pop()?;
                    match lhs - rhs {
                        Ok(val) => ex.stack.push(val),
                        Err(..) => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::Multiply => {
                    let rhs = ex.stack.pop()?;
                    let lhs = ex.stack.pop()?;
                    match lhs * rhs {
                        Ok(val) => ex.stack.push(val),
                        Err(..) => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::Divide => {
                    let rhs = ex.stack.pop()?;
                    let lhs = ex.stack.pop()?;
                    match lhs / rhs {
                        Ok(val) => ex.stack.push(val),
                        Err(..) => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::Remainder => {
                    let rhs = ex.stack.pop()?;
                    let lhs = ex.stack.pop()?;
                    match lhs % rhs {
                        Ok(val) => ex.stack.push(val),
                        Err(..) => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::IntDivide => {
                    let rhs = ex.stack.pop()?;
                    let lhs = ex.stack.pop()?;
                    match lhs / rhs {
                        Ok(Value::Number(val)) => {
                            ex.stack
                                .push(Value::Number(Number::from(val.as_complex().re.floor())));
                        }
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::Power => {
                    let rhs = ex.stack.pop()?;
                    let lhs = ex.stack.pop()?;
                    match (lhs, rhs) {
                        (Value::Number(..), Value::Number(..)) => todo!("surprisingly hard"),
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::Negate => {
                    let val = ex.stack.pop()?;
                    match -val {
                        Ok(val) => ex.stack.push(val),
                        Err(..) => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::Glue => {
                    let rhs = ex.stack.pop()?;
                    let lhs = ex.stack.pop()?;
                    match (lhs, rhs) {
                        (Value::String(lhs), Value::String(rhs)) => {
                            ex.stack.push(Value::String(lhs + &rhs))
                        }
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::Access => {
                    let rhs = ex.stack.pop()?;
                    let lhs = ex.stack.pop()?;
                    match (lhs, rhs) {
                        (Value::Record(record), rhs) => match record.get(&rhs) {
                            Some(value) => ex.stack.push(value),
                            None => todo!("yield 'MIA"),
                        },
                        (Value::String(lhs), Value::Number(rhs)) => {
                            let ch = rhs
                                .as_uinteger()
                                .and_then(|index| index.to_usize())
                                .and_then(|index| lhs.chars().nth(index));
                            match ch {
                                Some(ch) => ex.stack.push(Value::Char(ch)),
                                None => todo!("yield 'MIA"),
                            }
                        }
                        (Value::Bits(lhs), Value::Number(rhs)) => {
                            let val = rhs
                                .as_uinteger()
                                .and_then(|index| index.to_usize())
                                .and_then(|index| lhs.get(index));
                            match val {
                                Some(val) => ex.stack.push(Value::Bool(val)),
                                None => todo!("yield 'MIA"),
                            }
                        }
                        (Value::Array(lhs), Value::Number(rhs)) => {
                            let val = rhs
                                .as_uinteger()
                                .and_then(|index| index.to_usize())
                                .and_then(|index| lhs.get(index));
                            match val {
                                Some(val) => ex.stack.push(val),
                                None => todo!("yield 'MIA"),
                            }
                        }
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::Assign => {
                    let value = ex.stack.pop()?;
                    let rhs = ex.stack.pop()?;
                    let lhs = ex.stack.pop()?;
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
                OpCode::Not => {
                    let val = ex.stack.pop()?;
                    match val {
                        Value::Bool(val) => ex.stack.push(Value::Bool(!val)),
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::And => {
                    let rhs = ex.stack.pop()?;
                    let lhs = ex.stack.pop()?;
                    match (lhs, rhs) {
                        (Value::Bool(lhs), Value::Bool(rhs)) => {
                            ex.stack.push(Value::Bool(lhs && rhs))
                        }
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::Or => {
                    let rhs = ex.stack.pop()?;
                    let lhs = ex.stack.pop()?;
                    match (lhs, rhs) {
                        (Value::Bool(lhs), Value::Bool(rhs)) => {
                            ex.stack.push(Value::Bool(lhs || rhs))
                        }
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::BitwiseAnd => {
                    let lhs = ex.stack.pop()?;
                    let rhs = ex.stack.pop()?;
                    match lhs & rhs {
                        Ok(val) => ex.stack.push(val),
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::BitwiseOr => {
                    let lhs = ex.stack.pop()?;
                    let rhs = ex.stack.pop()?;
                    match lhs | rhs {
                        Ok(val) => ex.stack.push(val),
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::BitwiseXor => {
                    let lhs = ex.stack.pop()?;
                    let rhs = ex.stack.pop()?;
                    match lhs ^ rhs {
                        Ok(val) => ex.stack.push(val),
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::BitwiseNeg => {
                    let val = ex.stack.pop()?;
                    match val {
                        Value::Bits(val) => ex.stack.push(Value::Bits(!val)),
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::LeftShift => {
                    let lhs = ex.stack.pop()?;
                    let rhs = ex.stack.pop()?;
                    match lhs << rhs {
                        Ok(val) => ex.stack.push(val),
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::RightShift => {
                    let lhs = ex.stack.pop()?;
                    let rhs = ex.stack.pop()?;
                    match lhs >> rhs {
                        Ok(val) => ex.stack.push(val),
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::Cons => {
                    let lhs = ex.stack.pop()?;
                    let rhs = ex.stack.pop()?;
                    ex.stack.push(Value::Tuple(Tuple::new(lhs, rhs)));
                }
                OpCode::Leq => {
                    let lhs = ex.stack.pop()?;
                    let rhs = ex.stack.pop()?;
                    let cmp = match lhs.partial_cmp(&rhs) {
                        Some(Ordering::Less | Ordering::Equal) => Value::Bool(true),
                        Some(Ordering::Greater) => Value::Bool(false),
                        None => Value::Unit,
                    };
                    ex.stack.push(cmp);
                }
                OpCode::Lt => {
                    let lhs = ex.stack.pop()?;
                    let rhs = ex.stack.pop()?;
                    let cmp = match lhs.partial_cmp(&rhs) {
                        Some(Ordering::Less) => Value::Bool(true),
                        Some(Ordering::Greater) | Some(Ordering::Equal) => Value::Bool(false),
                        None => Value::Unit,
                    };
                    ex.stack.push(cmp);
                }
                OpCode::Geq => {
                    let lhs = ex.stack.pop()?;
                    let rhs = ex.stack.pop()?;
                    let cmp = match lhs.partial_cmp(&rhs) {
                        Some(Ordering::Less) => Value::Bool(false),
                        Some(Ordering::Greater) | Some(Ordering::Equal) => Value::Bool(true),
                        None => Value::Unit,
                    };
                    ex.stack.push(cmp);
                }
                OpCode::Gt => {
                    let lhs = ex.stack.pop()?;
                    let rhs = ex.stack.pop()?;
                    let cmp = match lhs.partial_cmp(&rhs) {
                        Some(Ordering::Less) | Some(Ordering::Equal) => Value::Bool(false),
                        Some(Ordering::Greater) => Value::Bool(true),
                        None => Value::Unit,
                    };
                    ex.stack.push(cmp);
                }
                OpCode::RefEq => {
                    let lhs = ex.stack.pop()?;
                    let rhs = ex.stack.pop()?;
                    ex.stack.push(Value::Bool(ReferentialEq::eq(&lhs, &rhs)));
                }
                OpCode::ValEq => {
                    let lhs = ex.stack.pop()?;
                    let rhs = ex.stack.pop()?;
                    ex.stack.push(Value::Bool(StructuralEq::eq(&lhs, &rhs)));
                }
                OpCode::RefNeq => {
                    let lhs = ex.stack.pop()?;
                    let rhs = ex.stack.pop()?;
                    ex.stack.push(Value::Bool(!ReferentialEq::eq(&lhs, &rhs)));
                }
                OpCode::ValNeq => {
                    let lhs = ex.stack.pop()?;
                    let rhs = ex.stack.pop()?;
                    ex.stack.push(Value::Bool(!StructuralEq::eq(&lhs, &rhs)));
                }
                OpCode::Call => {
                    let arity = ex.read_offset(&self.program.instructions)?;
                    let callable = ex.stack.replace_with_pointer(arity, ex.ip)?;
                    match callable {
                        Value::Continuation(continuation) => {
                            ex.call_continuation(continuation, arity)?;
                        }
                        Value::Procedure(ip) => ex.ip = ip,
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::Return => {
                    let return_value = ex.stack.pop()?;
                    let return_to = ex.stack.pop_pointer()?;
                    ex.ip = return_to;
                    ex.stack.push(return_value);
                }
                OpCode::Shift => {
                    let jump = ex.read_offset(&self.program.instructions)?;
                    let continuation = ex.current_continuation();
                    ex.stack.push(Value::Continuation(continuation));
                    ex.ip += jump;
                }
                OpCode::Reset => {
                    let return_value = ex.stack.pop()?;
                    ex.reset_continuation()?;
                    ex.stack.push(return_value);
                }
                OpCode::Jump => {
                    let dist = ex.read_offset(&self.program.instructions)?;
                    ex.ip += dist;
                }
                OpCode::JumpBack => {
                    let dist = ex.read_offset(&self.program.instructions)?;
                    ex.ip -= dist;
                }
                OpCode::CondJump => {
                    let dist = ex.read_offset(&self.program.instructions)?;
                    let cond = ex.stack.pop()?;
                    match cond {
                        Value::Bool(false) => ex.ip += dist,
                        Value::Bool(true) => {}
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::CondJumpBack => {
                    let dist = ex.read_offset(&self.program.instructions)?;
                    let cond = ex.stack.pop()?;
                    match cond {
                        Value::Bool(false) => ex.ip -= dist,
                        Value::Bool(true) => {}
                        _ => return Err(Error::RuntimeTypeError),
                    }
                }
                OpCode::Branch => {
                    // A branch requires two values on the stack; the two branches get the
                    // different values, respectively.
                    let right = ex.stack.pop()?;
                    let left = ex.stack.pop()?;
                    let mut branch = ex.branch();
                    ex.stack.push(left);
                    branch.stack.push(right);
                    self.executions.push_back(branch);
                }
                OpCode::Fizzle => {
                    // This just ends the execution.
                    //
                    // Is there any cleanup that has to be done? Or does Rust's
                    // RAII cause that cleanup to happen automatically?
                    //
                    // I suspect it is automatic.
                    self.executions.pop_front();
                }
                OpCode::Exit => {
                    // When run in embedded mode, the exit value can be any value. The
                    // interpreter binary can decide how to handle that exit value when
                    // passing off to the OS.
                    let value = ex.stack.pop()?;
                    self.executions.clear();
                    return Ok(value);
                }
            }
        }
        Err(Error::ExecutionFizzledError)
    }
}
