use super::stack::{StackDump, StackTrace};
use crate::bytecode::{ChunkError, Offset};
use crate::cactus::StackOverflow;
use crate::Value;
use std::fmt::{self, Display};

/// An error that has occurred during the execution of a program.
#[derive(Clone, Debug)]
pub struct Error {
    /// The instruction pointer at which the error occurred. Note that since the
    /// instruction was already read, this is actually the pointer to the instruction
    /// following the one that caused the error.
    pub ip: Offset,
    /// The type of error that occurred.
    pub kind: ErrorKind,
    /// The stack trace.
    pub stack_trace: StackTrace,
    /// A copy of the entire stack of the program at the time of this error.
    pub(crate) stack_dump: StackDump,
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::InvalidBytecode(error) => Some(error),
            ErrorKind::InternalRuntimeError(error) => Some(error),
            _ => None,
        }
    }
}

impl Error {
    pub fn dump(&self) -> &StackDump {
        &self.stack_dump
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

/// The types of errors that may occur when executing a program.
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug)]
pub enum ErrorKind {
    /// A runtime error explicitly raised by the program.
    ///
    /// This is the type of error produced by the `PANIC` instruction.
    RuntimeError(Value),
    /// All executions in the program have fizzled without reaching a suitable exit.
    ExecutionFizzledError,
    /// The code generator has produced something that cannot be compiled into valid bytecode.
    InvalidBytecode(ChunkError),
    /// The program stack has extended past its designated limit.
    StackOverflow,
    /// Something went wrong inside the virtual machine. Likely an issue with the compiler
    /// of the language which is being run, rather than with the execution of the program
    /// itself.
    InternalRuntimeError(InternalRuntimeError),
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RuntimeError(msg) => write!(f, "runtime error: {msg}"),
            Self::ExecutionFizzledError => write!(f, "execution fizzled"),
            Self::InvalidBytecode(error)  => write!(f, "{error}"),
            Self::StackOverflow => write!(f, "stack overflow"),
            Self::InternalRuntimeError(internal) => write!(f, "internal error ({internal})\nIf you are seeing this error, it is not likely something wrong with your program, but something wrong with the bytecode generated by the compiler. Please file a bug report to foxfriends/trilogy on GitHub."),
        }
    }
}

/// An error that occurred at runtime, internal to the virtual machine.
///
/// These are typically caused by improper bytecode being run. A "good" program should handle
/// all the errors that it possibly can such that these internal errors are never shown to
/// the end user, and instead raise RuntimeErrors whenever possible.
#[derive(Copy, Clone, Debug)]
pub enum InternalRuntimeError {
    /// A value was provided to the VM that was not of the expected type.
    TypeError,
    /// The generated bytecode contained an unknown opcode.
    InvalidOpcode(Offset, Offset),
    /// The generated bytecode referenced a register beyond the register limit.
    InvalidRegister(Offset),
    /// The requested offset is outside of the current stack range.
    OutOfStackRange(Offset),
    /// The generated bytecode references a constant that does not exist.
    MissingConstant,
    /// The generated bytecode attempts to use a pointer after it has been freed.
    UseAfterFree,
    /// A value was expected on the stack but something else was found.
    ExpectedValue(&'static str),
    /// A procedure's return pointer was expected on the stack but something else was found.
    ExpectedReturn,
}

impl std::error::Error for InternalRuntimeError {}

impl From<InternalRuntimeError> for ErrorKind {
    fn from(value: InternalRuntimeError) -> Self {
        Self::InternalRuntimeError(value)
    }
}

impl From<ChunkError> for ErrorKind {
    fn from(value: ChunkError) -> Self {
        Self::InvalidBytecode(value)
    }
}

impl From<StackOverflow> for ErrorKind {
    fn from(_: StackOverflow) -> Self {
        Self::StackOverflow
    }
}

impl Display for InternalRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeError => write!(f, "type error"),
            Self::InvalidOpcode(op, ip) => write!(f, "invalid opcode {op} at {ip}"),
            Self::InvalidRegister(reg) => write!(f, "invalid register {reg}"),
            Self::OutOfStackRange(index) => write!(f, "stack offset {index} out of range"),
            Self::MissingConstant => write!(f, "missing constant"),
            Self::UseAfterFree => write!(f, "use after free"),
            Self::ExpectedValue(found) => write!(f, "expected value on stack, found {found}"),
            Self::ExpectedReturn => write!(f, "expected return pointer on stack"),
        }
    }
}
