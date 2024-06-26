use crate::{Annotation, Atom, Instruction, Offset, Value};

/// Generic interface by which we can write to an underlying chunk.
///
/// This crate provides the [`ChunkBuilder`][crate::ChunkBuilder] as the primary implementer of this
/// trait, but your own crates may choose to implement this trait as well to make
/// use of common helper functions that need to take various levels of abstraction
/// around the writing of the chunk.
pub trait ChunkWriter {
    /// The IP of the line that is about to be written.
    ///
    /// Use this to get the information required to add annotations.
    fn ip(&self) -> Offset;

    /// Add an annotation to this chunk.
    ///
    /// Annotations have no effect on the program, but are used to provide meaningful information
    /// in stack traces, error messages, and debuggers.
    fn annotate(&mut self, annotation: Annotation) -> &mut Self;

    /// Add a label to the next instruction to be inserted.
    ///
    /// ```asm
    /// label:
    /// ```
    ///
    /// Note that if no instruction is inserted following this label, the label will
    /// be treated as if it was not defined.
    fn label<S: Into<String>>(&mut self, label: S) -> &mut Self;

    /// Protects the currently staged labels, ensuring they do not get stripped
    /// by any dead code elimination that may occur.
    ///
    /// It is important to protect any labels that future chunks may assume the existence
    /// of, as otherwise they might have already been removed.
    fn protect(&mut self) -> &mut Self;

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

    /// Insert a PJUMP instruction to a given label.
    ///
    /// ```asm
    /// PJUMP &label
    /// ```
    fn panic_jump<S: Into<String>>(&mut self, label: S) -> &mut Self;

    /// Insert a PJUMPF instruction to a given label.
    ///
    /// ```asm
    /// PJUMPF &label
    /// ```
    fn panic_cond_jump<S: Into<String>>(&mut self, label: S) -> &mut Self;

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

#[macro_export]
macro_rules! delegate_chunk_writer {
    ($t:ty, $f:ident) => {
        impl $crate::ChunkWriter for $t {
            fn ip(&self) -> $crate::Offset {
                self.$f.ip()
            }

            fn annotate(&mut self, annotation: $crate::Annotation) -> &mut Self {
                self.$f.annotate(annotation);
                self
            }

            fn reference<S: Into<String>>(&mut self, label: S) -> &mut Self {
                self.$f.reference(label);
                self
            }

            fn cond_jump<S: Into<String>>(&mut self, label: S) -> &mut Self {
                self.$f.cond_jump(label);
                self
            }

            fn panic_cond_jump<S: Into<String>>(&mut self, label: S) -> &mut Self {
                self.$f.panic_cond_jump(label);
                self
            }

            fn protect(&mut self) -> &mut Self {
                self.$f.protect();
                self
            }

            fn jump<S: Into<String>>(&mut self, label: S) -> &mut Self {
                self.$f.jump(label);
                self
            }

            fn panic_jump<S: Into<String>>(&mut self, label: S) -> &mut Self {
                self.$f.panic_jump(label);
                self
            }

            fn shift<S: Into<String>>(&mut self, label: S) -> &mut Self {
                self.$f.shift(label);
                self
            }

            fn close<S: Into<String>>(&mut self, label: S) -> &mut Self {
                self.$f.close(label);
                self
            }

            fn instruction(&mut self, instruction: $crate::Instruction) -> &mut Self {
                self.$f.instruction(instruction);
                self
            }

            fn label<S: Into<String>>(&mut self, label: S) -> &mut Self {
                self.$f.label(label);
                self
            }

            fn constant<V: Into<$crate::Value>>(&mut self, value: V) -> &mut Self {
                self.$f.constant(value);
                self
            }

            fn make_atom<S: AsRef<str>>(&self, value: S) -> $crate::Atom {
                self.$f.make_atom(value)
            }
        }
    };
}
