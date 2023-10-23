use crate::callable::Procedure;
use crate::Offset;
use crate::{atom::AtomInterner, Chunk, ChunkBuilder, ChunkError, Instruction, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A program that can be run on this VM.
///
/// The `Program` trait encapsulates the module resolution portion of a
/// particular language's runtime, allowing the relationship between
/// modules to be defined by the language.
///
/// # Examples
///
/// ```
/// # use trilogy_vm::{Program, VirtualMachine, Value, ChunkBuilder};
/// struct NoopProgram;
///
/// impl Program for NoopProgram {
///     fn chunk(&self, input: &Value, builder: &mut ChunkBuilder) {}
///
///     fn entrypoint(&self, builder: &mut ChunkBuilder) {
///         builder.parse(r#"
///             CONST 0
///             EXIT
///         "#);
///     }
/// }
///
/// # let vm = VirtualMachine::new();
/// assert_eq!(vm.run(&NoopProgram).unwrap(), Value::from(0));
/// ```
pub trait Program {
    /// Retrieve another chunk of code as described by a given value. The interpretation
    /// of the value (and production of the value) is at the language runtime's definition.
    fn chunk(&self, input: &Value, builder: &mut ChunkBuilder);

    /// Compute the initial chunk to execute when the virtual machine is provided with
    /// a new program.
    fn entrypoint(&self, builder: &mut ChunkBuilder);
}

pub(super) struct ProgramReader<'a> {
    program: &'a dyn Program,
    chunk: Arc<RwLock<Chunk>>,
    chunk_cache: Arc<RwLock<HashMap<Value, Value>>>,
    atom_interner: AtomInterner,
    entrypoint: Offset,
}

impl Clone for ProgramReader<'_> {
    fn clone(&self) -> Self {
        Self {
            program: self.program,
            chunk: self.chunk.clone(),
            chunk_cache: self.chunk_cache.clone(),
            atom_interner: self.atom_interner.clone(),
            entrypoint: self.entrypoint,
        }
    }
}

impl<'a> ProgramReader<'a> {
    pub(super) fn new(
        atom_interner: AtomInterner,
        program: &'a dyn Program,
    ) -> Result<Self, ChunkError> {
        let mut builder = ChunkBuilder::new(atom_interner.clone());
        program.entrypoint(&mut builder);
        let (entrypoint, chunk) = builder.build()?;
        Ok(Self {
            program,
            chunk: Arc::new(RwLock::new(chunk)),
            chunk_cache: Arc::default(),
            atom_interner,
            entrypoint,
        })
    }

    pub(super) fn read_instruction(&self, offset: u32) -> Instruction {
        Instruction::from_chunk(&self.chunk.read().unwrap(), offset)
    }

    pub(super) fn locate(&self, locator: Value) -> Result<Value, ChunkError> {
        let cache = self.chunk_cache.read().unwrap();
        match cache.get(&locator) {
            Some(cached) => Ok(cached.clone()),
            None => {
                std::mem::drop(cache);
                let mut chunk = self.chunk.write().unwrap();
                let mut builder = ChunkBuilder::new(self.atom_interner.clone());
                self.program.chunk(&locator, &mut builder);
                let entrypoint = builder.build_from(&mut chunk)?;
                let chunk_procedure = Value::from(Procedure::new(entrypoint));
                self.chunk_cache
                    .write()
                    .unwrap()
                    .insert(locator, chunk_procedure.clone());
                Ok(chunk_procedure)
            }
        }
    }

    pub(super) fn entrypoint(&self) -> Offset {
        self.entrypoint
    }
}
