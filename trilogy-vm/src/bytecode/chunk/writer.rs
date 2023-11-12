use crate::{Atom, Instruction, Value};

/// Generic interface by which we can write to an underlying chunk.
///
/// This crate provides the [`ChunkBuilder`][] as the primary implementer of this
/// trait, but your own crates may choose to implement this trait as well to make
/// use of common helper functions that need to take various levels of abstraction
/// around the writing of the chunk.
pub trait ChunkWriter {
    /// Add a label to the next instruction to be inserted.
    ///
    /// ```asm
    /// label:
    /// ```
    ///
    /// Note that if no instruction is inserted following this label, the label will
    /// be treated as if it was not defined.
    fn label<S: Into<String>>(&mut self, label: S) -> &mut Self;

    /// Insert a CONST instruction that references a procedure located at the
    /// given label.
    ///
    /// ```asm
    /// CONST &label
    /// ```
    fn reference<S: Into<String>>(&mut self, label: S) -> &mut Self;

    /// Insert a JUMP instruction to a given label.
    ///
    /// ```asm
    /// JUMP &label
    /// ```
    fn jump<S: Into<String>>(&mut self, label: S) -> &mut Self;

    /// Insert a JUMPF instruction to a given label.
    ///
    /// ```asm
    /// JUMPF &label
    /// ```
    fn cond_jump<S: Into<String>>(&mut self, label: S) -> &mut Self;

    /// Insert a CLOSE instruction to a given label.
    ///
    /// ```asm
    /// CLOSE &label
    /// ```
    fn close<S: Into<String>>(&mut self, label: S) -> &mut Self;

    /// Insert a SHIFT instruction to a given label.
    ///
    /// ```asm
    /// SHIFT &label
    /// ```
    fn shift<S: Into<String>>(&mut self, label: S) -> &mut Self;

    /// Insert an instruction.
    ///
    /// All labels currently in the buffer will be assigned to this line, and
    /// the buffer will be cleared.
    fn instruction(&mut self, instruction: Instruction) -> &mut Self;

    /// Instantiate an atom for the current runtime. Atoms cannot be created except
    /// for within the context of a particular runtime's global atom table.
    fn make_atom<S: AsRef<str>>(&self, atom: S) -> Atom;

    /// Insert a CONST instruction of the given value.
    fn constant<V: Into<Value>>(&mut self, value: V) -> &mut Self {
        self.instruction(Instruction::Const(value.into()))
    }

    /// Insert a CONST instruction where the value is created by converting the given string to
    /// an atom.
    fn atom<S: AsRef<str>>(&mut self, atom: S) -> &mut Self {
        let atom = self.make_atom(atom);
        self.constant(atom)
    }
}
