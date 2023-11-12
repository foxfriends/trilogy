use crate::INCORRECT_ARITY;
use crate::INVALID_CALL;
use crate::INVALID_ITERATOR;
use crate::RUNTIME_TYPE_ERROR;
pub(crate) use trilogy_vm::ChunkWriter;
pub(crate) use trilogy_vm::Instruction;
use trilogy_vm::Struct;
use trilogy_vm::Value;

pub(crate) trait TypePattern {
    fn write<W: ChunkWriter + LabelMaker>(&self, writer: &mut W, destination: Result<&str, &str>);
}

impl TypePattern for () {
    fn write<W: ChunkWriter + LabelMaker>(
        &self,
        _writer: &mut W,
        _destination: Result<&str, &str>,
    ) {
    }
}

impl TypePattern for str {
    fn write<W: ChunkWriter + LabelMaker>(&self, writer: &mut W, destination: Result<&str, &str>) {
        writer
            .instruction(Instruction::Copy)
            .instruction(Instruction::TypeOf)
            .atom(self);
        match destination {
            Ok(destination) => writer
                .instruction(Instruction::ValNeq)
                .cond_jump(destination),
            Err(destination) => writer
                .instruction(Instruction::ValEq)
                .cond_jump(destination),
        };
    }
}

impl TypePattern for [&str] {
    fn write<W: ChunkWriter + LabelMaker>(&self, writer: &mut W, destination: Result<&str, &str>) {
        match destination {
            Ok(destination) => {
                let done = writer.make_label("done");
                for t in self {
                    t.write(writer, Err(&done));
                }
                writer.jump(destination).label(done);
            }
            Err(destination) => {
                let done = writer.make_label("done");
                for t in self {
                    t.write(writer, Ok(&done));
                }
                writer.jump(destination).label(done);
            }
        }
    }
}

pub(crate) trait ChunkWriterExt: ChunkWriter + LabelMaker + Sized {
    fn r#struct<V: Into<Value>>(&mut self, atom: &str, value: V) -> &mut Self {
        let atom = self.make_atom(atom);
        self.constant(Struct::new(atom, value.into()))
    }

    fn typecheck<T>(&mut self, types: &T) -> &mut Self
    where
        T: TypePattern + ?Sized,
    {
        types.write(self, Err(RUNTIME_TYPE_ERROR));
        self
    }

    fn try_type<T>(&mut self, types: &T, destination: Result<&str, &str>) -> &mut Self
    where
        T: TypePattern + ?Sized,
    {
        types.write(self, destination);
        self
    }

    fn unlock_call(&mut self, atom: &str, arity: usize) -> &mut Self {
        self.instruction(Instruction::Destruct)
            .instruction(Instruction::Copy)
            .atom(atom)
            .instruction(Instruction::ValEq)
            .cond_jump(INVALID_CALL)
            .instruction(Instruction::Pop)
            .instruction(Instruction::Copy)
            .constant(arity)
            .instruction(Instruction::ValEq)
            .cond_jump(INCORRECT_ARITY)
            .instruction(Instruction::Pop)
    }

    fn unlock_function(&mut self) -> &mut Self {
        self.unlock_call("function", 1)
    }

    fn unlock_module(&mut self) -> &mut Self {
        self.unlock_call("module", 1)
    }

    fn unlock_procedure(&mut self, arity: usize) -> &mut Self {
        self.unlock_call("procedure", arity)
    }

    fn unlock_rule(&mut self, arity: usize) -> &mut Self {
        self.unlock_call("rule", arity)
    }

    fn call_procedure(&mut self, arity: usize) -> &mut Self {
        self.r#struct("procedure", arity)
            .instruction(Instruction::Call(arity as u32 + 1))
    }

    fn call_function(&mut self) -> &mut Self {
        self.r#struct("function", 1)
            .instruction(Instruction::Call(2))
    }

    fn call_rule(&mut self, arity: usize) -> &mut Self {
        self.r#struct("rule", arity)
            .instruction(Instruction::Call(arity as u32 + 1))
    }

    fn become_function(&mut self) -> &mut Self {
        self.r#struct("function", 1)
            .instruction(Instruction::Become(2))
    }

    fn call_module(&mut self) -> &mut Self {
        self.r#struct("module", 1).instruction(Instruction::Call(2))
    }

    fn is_done<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.instruction(Instruction::Copy)
            .atom("done")
            .instruction(Instruction::ValNeq)
            .cond_jump(label)
    }

    fn unwrap_next(&mut self) -> &mut Self {
        self.instruction(Instruction::Copy)
            .instruction(Instruction::TypeOf)
            .atom("struct")
            .instruction(Instruction::ValEq)
            .cond_jump(INVALID_ITERATOR)
            .instruction(Instruction::Destruct)
            .atom("next")
            .instruction(Instruction::ValEq)
            .cond_jump(INVALID_ITERATOR)
    }

    fn iterate<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.instruction(Instruction::Call(0))
            .is_done(label)
            .unwrap_next()
    }
}

impl<T> ChunkWriterExt for T where T: ChunkWriter + LabelMaker {}

pub(crate) trait LabelMaker {
    fn make_label(&mut self, label: &str) -> String;
}