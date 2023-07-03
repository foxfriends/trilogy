use crate::bytecode::OpCode;
use crate::runtime::atom::AtomInterner;
use crate::runtime::Bits;
use crate::{Instruction, Program, Struct, Tuple, Value};
use std::str::FromStr;

#[derive(Copy, Clone, Debug)]
pub struct Error {
    pub line: usize,
    pub error: InvalidInstruction,
}

impl FromStr for Program {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut interner = AtomInterner::default();
        let mut program = Program {
            constants: vec![],
            instructions: vec![],
        };
        let mut writer = ProgramWriter {
            program: &mut program,
        };

        let instructions = s
            .lines()
            .map(|line| Instruction::parse(line, &mut interner))
            .enumerate()
            .map(|(line, result)| result.map_err(|error| Error { line, error }));

        for instruction in instructions {
            match instruction? {
                Instruction::Const(constant) => {
                    let index = writer.add_constant(constant);
                    writer.write_opcode(OpCode::Const);
                    writer.write_offset(index);
                }
                Instruction::Load(offset) => {
                    writer.write_opcode(OpCode::Load);
                    writer.write_offset(offset);
                }
                Instruction::Set(offset) => {
                    writer.write_opcode(OpCode::Set);
                    writer.write_offset(offset);
                }
                Instruction::Pop => writer.write_opcode(OpCode::Pop),
                Instruction::Add => writer.write_opcode(OpCode::Add),
                Instruction::Subtract => writer.write_opcode(OpCode::Subtract),
                Instruction::Multiply => writer.write_opcode(OpCode::Multiply),
                Instruction::Divide => writer.write_opcode(OpCode::Divide),
                Instruction::Remainder => writer.write_opcode(OpCode::Remainder),
                Instruction::IntDivide => writer.write_opcode(OpCode::IntDivide),
                Instruction::Power => writer.write_opcode(OpCode::Power),
                Instruction::Negate => writer.write_opcode(OpCode::Negate),
                Instruction::Glue => writer.write_opcode(OpCode::Glue),
                Instruction::Access => writer.write_opcode(OpCode::Access),
                Instruction::Assign => writer.write_opcode(OpCode::Assign),
                Instruction::Not => writer.write_opcode(OpCode::Not),
                Instruction::And => writer.write_opcode(OpCode::And),
                Instruction::Or => writer.write_opcode(OpCode::Or),
                Instruction::BitwiseAnd => writer.write_opcode(OpCode::BitwiseAnd),
                Instruction::BitwiseOr => writer.write_opcode(OpCode::BitwiseOr),
                Instruction::BitwiseXor => writer.write_opcode(OpCode::BitwiseXor),
                Instruction::BitwiseNeg => writer.write_opcode(OpCode::BitwiseNeg),
                Instruction::LeftShift => writer.write_opcode(OpCode::LeftShift),
                Instruction::RightShift => writer.write_opcode(OpCode::RightShift),
                Instruction::Cons => writer.write_opcode(OpCode::Cons),
                Instruction::Leq => writer.write_opcode(OpCode::Leq),
                Instruction::Lt => writer.write_opcode(OpCode::Lt),
                Instruction::Geq => writer.write_opcode(OpCode::Geq),
                Instruction::Gt => writer.write_opcode(OpCode::Gt),
                Instruction::RefEq => writer.write_opcode(OpCode::RefEq),
                Instruction::ValEq => writer.write_opcode(OpCode::ValEq),
                Instruction::RefNeq => writer.write_opcode(OpCode::RefNeq),
                Instruction::ValNeq => writer.write_opcode(OpCode::ValNeq),
                Instruction::Call(offset) => {
                    writer.write_opcode(OpCode::Call);
                    writer.write_offset(offset);
                }
                Instruction::Return => writer.write_opcode(OpCode::Return),
                Instruction::Shift(offset) => {
                    writer.write_opcode(OpCode::Shift);
                    writer.write_offset(offset);
                }
                Instruction::Reset => writer.write_opcode(OpCode::Reset),
                Instruction::Jump(offset) => {
                    writer.write_opcode(OpCode::Jump);
                    writer.write_offset(offset);
                }
                Instruction::JumpBack(offset) => {
                    writer.write_opcode(OpCode::JumpBack);
                    writer.write_offset(offset);
                }
                Instruction::CondJump(offset) => {
                    writer.write_opcode(OpCode::CondJump);
                    writer.write_offset(offset);
                }
                Instruction::CondJumpBack(offset) => {
                    writer.write_opcode(OpCode::CondJumpBack);
                    writer.write_offset(offset);
                }
                Instruction::Branch => writer.write_opcode(OpCode::Branch),
                Instruction::Fizzle => writer.write_opcode(OpCode::Fizzle),
                Instruction::Exit => writer.write_opcode(OpCode::Exit),
            }
        }

        Ok(program)
    }
}

struct ProgramWriter<'a> {
    program: &'a mut Program,
}

impl ProgramWriter<'_> {
    fn add_constant(&mut self, constant: Value) -> usize {
        let index = self.program.constants.len();
        self.program.constants.push(constant);
        index
    }

    fn write_opcode(&mut self, opcode: OpCode) {
        self.program.instructions.push(opcode as u8);
    }

    fn write_offset(&mut self, offset: usize) {
        self.program
            .instructions
            .extend((offset as u32).to_be_bytes())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ValueError {
    InvalidProcedure,
    InvalidCharacter,
    InvalidAtom,
    InvalidTuple,
    InvalidNumber,
    InvalidStruct,
    InvalidArray,
    InvalidString,
    InvalidRecord,
    InvalidSet,
    ExtraChars,
}

impl Value {
    fn parse_prefix<'a>(
        s: &'a str,
        interner: &mut AtomInterner,
    ) -> Result<(Self, &'a str), ValueError> {
        match s {
            _ if s.starts_with("unit") => Ok((Value::Unit, &s[4..])),
            _ if s.starts_with("true") => Ok((Value::Bool(true), &s[4..])),
            _ if s.starts_with("false") => Ok((Value::Bool(false), &s[5..])),
            _ if s.starts_with('\'') => {
                if s.starts_with('\\') {
                    todo!("Escape sequence")
                } else if &s[2..3] == "'" {
                    Ok((
                        Value::Char(s[1..2].parse().map_err(|_| ValueError::InvalidCharacter)?),
                        &s[3..],
                    ))
                } else {
                    let atom: String = s[1..]
                        .chars()
                        .take_while(|&ch| ch.is_ascii_alphanumeric() || ch == '_')
                        .collect();
                    if atom.is_empty() {
                        Err(ValueError::InvalidAtom)
                    } else {
                        let s = &s[atom.len()..];
                        let atom = interner.intern(&atom);
                        if let Some(s) = s.strip_prefix('(') {
                            let (value, s) = Value::parse_prefix(s, interner)?;
                            let s = s.strip_prefix(')').ok_or(ValueError::InvalidStruct)?;
                            Ok((Value::Struct(Struct::new(atom, value)), s))
                        } else {
                            Ok((Value::Atom(atom), s))
                        }
                    }
                }
            }
            _ if s.starts_with('(') => {
                let s = &s[1..];
                let (lhs, s) = Value::parse_prefix(s, interner)?;
                let s = s.strip_prefix(':').ok_or(ValueError::InvalidTuple)?;
                let (rhs, s) = Value::parse_prefix(s, interner)?;
                let s = s.strip_prefix(')').ok_or(ValueError::InvalidTuple)?;
                Ok((Value::Tuple(Tuple::new(lhs, rhs)), s))
            }
            _ if s.starts_with('"') => todo!(),
            _ if s.starts_with("[|") => todo!(),
            _ if s.starts_with("{|") => todo!(),
            _ if s.starts_with('[') => todo!(),
            _ if s.starts_with('&') => {
                let s = &s[1..];
                let numberlike: String = s.chars().take_while(|ch| ch.is_numeric()).collect();
                let offset = numberlike
                    .parse()
                    .map_err(|_| ValueError::InvalidProcedure)?;
                Ok((Value::Procedure(offset), &s[numberlike.len()..]))
            }
            _ if s.starts_with("0b") => {
                let bits: Bits = s[2..]
                    .chars()
                    .take_while(|&c| c == '0' || c == '1')
                    .map(|ch| ch == '1')
                    .collect();
                let s = &s[bits.len() + 2..];
                Ok((Value::Bits(bits), s))
            }
            s => {
                let numberlike: String = s
                    .chars()
                    .take_while(|&c| {
                        // All the valid characters of these complex big rationals
                        c.is_numeric()
                            || c == '-'
                            || c == '+'
                            || c == 'i'
                            || c == 'e'
                            || c == '.'
                            || c == 'E'
                            || c == '/'
                    })
                    .collect();
                Ok((
                    Value::Number(numberlike.parse().map_err(|_| ValueError::InvalidNumber)?),
                    &s[numberlike.len()..],
                ))
            }
        }
    }

    fn parse(s: &str, interner: &mut AtomInterner) -> Result<Self, ValueError> {
        match Value::parse_prefix(s, interner) {
            Ok((value, "")) => Ok(value),
            Ok(..) => Err(ValueError::ExtraChars),
            Err(error) => Err(error),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum InvalidInstruction {
    UnknownOpcode,
    MissingParameter,
    InvalidOffset,
    InvalidValue(ValueError),
}

impl Instruction {
    fn parse(s: &str, interner: &mut AtomInterner) -> Result<Self, InvalidInstruction> {
        let (opcode, param) = s
            .split_once(' ')
            .map(|(s, p)| (s, Some(p)))
            .unwrap_or((s, None));

        fn offset(param: Option<&str>) -> Result<usize, InvalidInstruction> {
            param
                .ok_or(InvalidInstruction::MissingParameter)
                .and_then(|param| param.parse().map_err(|_| InvalidInstruction::InvalidOffset))
        }

        fn value(
            param: Option<&str>,
            interner: &mut AtomInterner,
        ) -> Result<Value, InvalidInstruction> {
            param
                .ok_or(InvalidInstruction::MissingParameter)
                .and_then(|param| {
                    Value::parse(param, interner).map_err(InvalidInstruction::InvalidValue)
                })
        }

        match opcode {
            "CONST" => Ok(Self::Const(value(param, interner)?)),
            "LOAD" => Ok(Self::Load(offset(param)?)),
            "SET" => Ok(Self::Set(offset(param)?)),
            "POP" => Ok(Self::Pop),
            "ADD" => Ok(Self::Add),
            "SUB" => Ok(Self::Subtract),
            "MUL" => Ok(Self::Multiply),
            "DIV" => Ok(Self::Divide),
            "REM" => Ok(Self::Remainder),
            "INTDIV" => Ok(Self::IntDivide),
            "POW" => Ok(Self::Power),
            "NEG" => Ok(Self::Negate),
            "GLUE" => Ok(Self::Glue),
            "ACCESS" => Ok(Self::Access),
            "ASSIGN" => Ok(Self::Assign),
            "NOT" => Ok(Self::Not),
            "AND" => Ok(Self::And),
            "OR" => Ok(Self::Or),
            "BITAND" => Ok(Self::BitwiseAnd),
            "BITOR" => Ok(Self::BitwiseOr),
            "BITXOR" => Ok(Self::BitwiseXor),
            "BITNEG" => Ok(Self::BitwiseNeg),
            "LSHIFT" => Ok(Self::LeftShift),
            "RSHIFT" => Ok(Self::RightShift),
            "CONS" => Ok(Self::Cons),
            "LEQ" => Ok(Self::Leq),
            "LT" => Ok(Self::Lt),
            "GEQ" => Ok(Self::Geq),
            "GT" => Ok(Self::Gt),
            "REFEQ" => Ok(Self::RefEq),
            "VALEQ" => Ok(Self::ValEq),
            "REFNEQ" => Ok(Self::RefNeq),
            "VALNEQ" => Ok(Self::ValNeq),
            "CALL" => Ok(Self::Call(offset(param)?)),
            "RETURN" => Ok(Self::Return),
            "SHIFT" => Ok(Self::Shift(offset(param)?)),
            "RESET" => Ok(Self::Reset),
            "JUMP" => Ok(Self::Jump(offset(param)?)),
            "RJUMP" => Ok(Self::JumpBack(offset(param)?)),
            "JUMPF" => Ok(Self::CondJump(offset(param)?)),
            "RJUMPF" => Ok(Self::CondJumpBack(offset(param)?)),
            "BRANCH" => Ok(Self::Branch),
            "FIZZLE" => Ok(Self::Fizzle),
            "EXIT" => Ok(Self::Exit),
            _ => Err(InvalidInstruction::UnknownOpcode),
        }
    }
}
