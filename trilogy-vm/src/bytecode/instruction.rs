use super::chunk::Chunk;
use crate::Value;
use trilogy_vm_derive::Asm;

/// Integer type used as the single parameter to some instructions.
#[cfg(not(any(feature = "64bit", feature = "32bit")))]
pub type Offset = usize;

/// Integer type used as the single parameter to some instructions.
#[cfg(all(feature = "64bit", not(feature = "32bit")))]
pub type Offset = u64;

/// Integer type used as the single parameter to some instructions.
#[cfg(feature = "32bit")]
pub type Offset = u32;

#[cfg(all(feature = "32bit", feature = "64bit"))]
compile_error!("Exactly one of the features `32bit` or `64bit` may be specified at one time.");

/// An instruction for the Trilogy VM.
///
/// In bytecode form, an instruction is represented as a single-byte [`OpCode`][].
/// Some op-codes are followed by single integer parameter, whose interpretation
/// is different depending on the specific instruction.
#[rustfmt::skip]
#[derive(Debug, Asm)]
#[cfg_attr(not(any(feature = "64bit", feature = "32bit")), asm(derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug), repr(usize)))]
#[cfg_attr(all(feature = "64bit", not(feature = "#2bit")), asm(derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug), repr(u64)))]
#[cfg_attr(feature = "32bit", asm(derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug), repr(u32)))]
pub enum Instruction {
    // Stack
    /// Push a constant value to the top of the stack
    Const(Value),
    /// Push a second copy of the value currently on the top of the stack.
    /// This is a shallow copy, so it will be referentially equal to the
    /// previous value if it is of a reference type.
    Copy,
    /// Replace the value on the top of the stack with a shallow clone
    /// of itself.
    ///
    /// The new value will not be referentially equal to the previous value if it was
    /// of a compound reference type. Callable types will remain referentially equal,
    /// as they cannot be further cloned.
    Clone,
    /// Replace the value on the top of the stack with a structural clone
    /// of itself.
    ///
    /// The new value will not be referentially equal to the previous value if it was
    /// of a compound reference type. Callable types will remain referentially equal,
    /// as they cannot be further cloned.
    #[asm(name = "CLONED")]
    DeepClone,
    /// Remove the value from the top of the stack.
    Pop,
    /// Swap the value on the top of the stack with the second from the top value.
    Swap,
    /// Slide the value on the top of the stack a number of places backwards.
    Slide(Offset),
    /// Replace the value on the top of the stack with an atom value representing
    /// the value's type. The atom will be one of:
    /// * `'unit`
    /// * `'number`
    /// * `'bits`
    /// * `'boolean`
    /// * `'string`
    /// * `'character`
    /// * `'tuple`
    /// * `'array`
    /// * `'set`
    /// * `'record`
    /// * `'atom`
    /// * `'struct`
    /// * `'callable`
    TypeOf,

    // Heap (why?)
    // /// Replace the pointer at the top of the stack with the value to which it points,
    // /// from the heap.
    // #[asm(name = "LOAD")] Load,
    // /// Set the value pointed to by the pointer at the top of the stack with the value
    // /// that is stored second from the top of the stack.
    // #[asm(name = "SET")] Set,
    // /// Initialize the value pointed to by the pointer at the top of the stack with
    // /// the value that is stored second from the top of the stack. Pushes a boolean
    // /// value: `true` if the initialization succeeded, or `false` if the initialization
    // /// failed due to the pointer having already been set.
    // #[asm(name = "INIT")] Init,
    // /// Unsets the value pointed to by the pointer at the top of the stack.
    // #[asm(name = "UNSET")] Unset,
    // /// Check whether the value pointed to by the pointer at the top of the stack is set
    // /// or not.
    // #[asm(name = "ISSET")] IsSet,
    // /// Allocates a new pointer, pushing the pointer to the top of the stack. The pointer
    // /// initially points to an empty slot on the heap.
    // Alloc,
    // /// Frees the slot on the heap pointed to by the pointer at the top of the stack. It
    // /// is invalid to use a pointer after it has been freed.
    // Free,

    // Variables

    /// Make space for a variable on the top of the stack. This space is initially empty.
    #[asm(name = "VAR")] Variable,
    /// Push the value from the "local variable" at the given offset to the top of the stack.
    /// This offset is relative to the nearest call frame.
    #[asm(name = "LOADL")] LoadLocal(Offset),
    /// Set the value of the "local variable" at the given offset to the value at the top
    /// of the stack. This offset is relative to the nearest call frame.
    #[asm(name = "SETL")] SetLocal(Offset),
    /// Initialize the value of a "local variable" at the given offset to the value at the
    /// top of the stack. Pushes a boolean value: `true` if the initialization succeeded, or
    /// `false` if the initialization failed due to the variable having already been set.
    #[asm(name = "INITL")] InitLocal(Offset),
    /// Unset the value of the "local variable" at the given offset.
    #[asm(name = "UNSETL")] UnsetLocal(Offset),
    /// Check whether the "local variable" at the given offset is set or not.
    #[asm(name = "ISSETL")] IsSetLocal(Offset),
    /// Load the value from a given register.
    ///
    /// Registers referenced by the bytecode must have been provided by the host program.
    #[asm(name = "LOADR")] LoadRegister(Offset),
    /// Set the value of a given register.
    ///
    /// Registers referenced by the bytecode must have been provided by the host program.
    #[asm(name = "SETR")] SetRegister(Offset),

    // Numbers
    /// Add the top two values from the stack, pushing their sum.
    Add,
    /// Subtract the top value from the stack from the second from the top value, pushing
    /// the difference.
    #[asm(name = "SUB")] Subtract,
    /// Multiply the top two values from the stack, pushing their product.
    #[asm(name = "MUL")] Multiply,
    /// Divide the second from the top value from the stack by the top value, pushing the
    /// quotient.
    #[asm(name = "DIV")] Divide,
    /// Divide the second from the top value from the stack by the top value, pushing the
    /// remainder.
    #[asm(name = "REM")] Remainder,
    /// Divide the second from the top value from the stack by the top value, pushing the
    /// quotient rounded towards negative infinity to the nearest integer.
    #[asm(name = "INTDIV")] IntDivide,
    /// Raise the second from the top value from the stack to the power of the top value from
    /// the stack, pushing the result.
    #[asm(name = "POW")] Power,
    /// Pop the value from the top of the stack, pushing its negation.
    #[asm(name = "NEG")] Negate,

    // Collections
    /// Using the top value from the stack as the key, access the value at that key from the
    /// collection at the second from the top of the stack, pushing the value that was found.
    Access,
    /// Set the value of the collection stored at the third spot from the top of the stack
    /// at the key stored at the second spot from the top of the stack to the value from the
    /// top of the stack. The extended collection is pushed back onto the stack.
    Assign,
    /// Insert the value from the top of the stack into the collection second from the top
    /// of the stack. The extended collection is pushed back onto the stack.
    Insert,
    /// Delete the value stored at the key at the top of the stack from the collection second
    /// from the top of the stack. The collection is pushed back onto the stack.
    Delete,
    /// Check if the key at the top of the stack is found within the collection second from the
    /// top of a stack. Pushes the boolean result.
    Contains,
    /// Convert the collection at the top of the stack to an array of its entries.
    Entries,
    /// Replace the collection at the top of the stack with the number of elements that collection
    /// contained.
    Length,
    /// Using the number at the top of the stack as the number of elements, take that many elements
    /// from the front of the collection second from the top of the stack, pushing that slice back
    /// onto the stack.
    Take,
    /// Using the number at the top of the stack as the number of elements, skip that many elements
    /// from the front of collection second from the top of the stack, pushing the rest of the
    /// collection back onto the stack.
    Skip,
    /// Glue the collection on the top of the stack onto the end of the collection second from the
    /// top of the stack.
    Glue,

    // Booleans
    /// Pop the value from the top of the stack, pushing the inverted boolean.
    Not,
    /// Pop two booleans from the top of the stack, and push true if both are true, or false otherwise.
    And,
    /// Pop two booleans from the top of the stack, and push true if either is true, or false otherwise.
    Or,

    // Bits
    /// Compute the bitwise "AND" operation of the two bits values from the top of the stack,
    /// pushing the result.
    #[asm(name = "BITAND")] BitwiseAnd,
    /// Compute the bitwise "OR" operation of the two bits values from the top of the stack,
    /// pushing the result.
    #[asm(name = "BITOR")] BitwiseOr,
    /// Compute the bitwise "XOR" operation of the two bits values from the top of the stack,
    /// pushing the result.
    #[asm(name = "BITXOR")] BitwiseXor,
    /// Pop a bits value from the top o the stack, pushing the inversion of that value.
    #[asm(name = "BITNEG")] BitwiseNeg,
    /// Shift all the bits of the bits value second from the top of the stack to the left by the
    /// number of places at the top of the stack, pushing the result.
    #[asm(name = "BITSHIFTL")] LeftShift,
    /// Shift all the bits of the bits value second from the top of the stack to the right
    /// by the number of places at the top of the stack, pushing the result.
    #[asm(name = "BITSHIFTR")] RightShift,

    // Tuples
    /// Construct a tuple of the value second from the top of the stack as the first element, and the
    /// value at the top of the stack as the second.
    Cons,
    /// Deconstruct a tuple, pushing the first value and then the second to the top of the stack.
    Uncons,
    /// Pop a tuple from the stack, and push its first value.
    First,
    /// Pop a tuple from the stack, and push its second value.
    Second,

    // Structs
    /// Wrap the value at the top of the stack in a struct named by the atom second from
    /// the top of the stack, pushing the constructed struct.
    Construct,
    /// Take apart the struct from the top of the stack, pushing its tag atom followed
    /// by the wrapped value.
    Destruct,

    // Comparison
    /// Pop two values from the top of the stack, pushing `true` if the second is less than or
    /// equal to the first, or `false` otherwise.
    Leq,
    /// Pop two values from the top of the stack, pushing `true` if the second is less than
    /// the first, or `false` otherwise.
    Lt,
    /// Pop two values from the top of the stack, pushing `true` if the second is greater than or
    /// equal to the first, or `false` otherwise.
    Geq,
    /// Pop two values from the top of the stack, pushing `true` if the second is greater than
    /// the first, or `false` otherwise.
    Gt,
    /// Pop two values from the top of the stack, pushing `true` if they are referentially equal,
    /// or `false` otherwise.
    RefEq,
    /// Pop two values from the top of the stack, pushing `true` if they are structurally equal,
    /// or `false` otherwise.
    ValEq,
    /// Pop two values from the top of the stack, pushing `true` if they are referentially inequal,
    /// or `false` otherwise.
    RefNeq,
    /// Pop two values from the top of the stack, pushing `true` if they are structurally inequal,
    /// or `false` otherwise.
    ValNeq,

    // Control Flow
    /// "Call" a callable value with the number of arguments specified in the offset. The stack
    /// should contain the callable value followed by that many values which will be used as the
    /// arguments.
    ///
    /// Calling the callable involves jumping to the instruction pointer to which that value points,
    /// and then pushing the callable value's stack frame.
    Call(Offset),
    /// Call the a callable value with the number of arguments specified in the offset. The stack
    /// should contain the callable value followed by that many values which will be used as the
    /// arguments.
    ///
    /// Becoming the callable involves jumping to the instruction pointer to which that value points,
    /// and then replacing the current stack frame with the callable value's stack frame.
    Become(Offset),
    /// Return the instruction pointer to the current frame's return pointer, ending the call.
    /// The value at the top of the stack is used as the return value, which is pushed to the top
    /// of the stack after returning.
    Return,
    /// Create a closure of the current stack frame. Control is tranferred to the offset specified.
    ///
    /// The closure is pushed to the top of the stack as a callable value, which progresses from the
    /// current frame when called.
    Close(Offset),
    /// Capture the current continuation. Control is transferred to the offset specified.
    ///
    /// The continuation is pushed to the top of the stack as a callable value, which continues from
    /// the current continuation when called.
    Shift(Offset),
    /// Jump to the offset specified.
    Jump(Offset),
    /// Pop the top value from the stack. If it is `false`, jump to the offset specified.
    #[asm(name = "JUMPF")] CondJump(Offset),
    /// Pop the top two values from the top of the stack, then split the current execution into two.
    /// The value from the top of the stack is pushed onto the new execution, and the second value
    /// from the top of the stack is pushed onto the previously existing execution. Both executions
    /// will continue "in parallel," whatever that means.
    Branch,
    /// End the current execution. If this was the only execution, the program ends in failure.
    Fizzle,
    /// End the current program. The value from the top of the stack is used as the exit value of
    /// the program.
    Exit,
    /// End the current program in error. The value from the top of the stack is used as the
    /// error value.
    Panic,

    // Meta
    /// Using the value as its identifier, load the corresponding chunk from the program currently
    /// being executed. Any value may be used, so long as the [`Program`][crate::Program]
    /// implementation is able to handle it.
    Chunk(Value),
    /// Print the value currently on the top of the stack to stderr using a
    /// debug representation. The value is not removed from the stack.
    Debug,
}

impl TryFrom<Offset> for OpCode {
    type Error = Offset;

    fn try_from(value: Offset) -> Result<Self, Self::Error> {
        if value <= Self::Debug as Offset {
            Ok(unsafe { std::mem::transmute(value) })
        } else {
            Err(value)
        }
    }
}

trait FromChunk {
    fn from_chunk(chunk: &Chunk, offset: Offset) -> Self;
}

impl FromChunk for Offset {
    #[inline(always)]
    fn from_chunk(_: &Chunk, offset: Offset) -> Self {
        offset
    }
}

impl FromChunk for Value {
    #[inline(always)]
    fn from_chunk(chunk: &Chunk, offset: Offset) -> Self {
        chunk.constant(offset)
    }
}
