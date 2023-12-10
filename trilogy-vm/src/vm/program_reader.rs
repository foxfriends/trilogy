use crate::callable::Procedure;
use crate::{atom::AtomInterner, Chunk, ChunkBuilder, ChunkError, Instruction, Value};
use crate::{Annotation, Offset, Program, RefCount};
use std::collections::HashMap;
use std::sync::RwLock;

pub(super) struct ProgramReader<'a> {
    program: &'a dyn Program,
    chunk: RefCount<RwLock<Chunk>>,
    chunk_cache: RefCount<RwLock<HashMap<Value, Value>>>,
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
            chunk: RefCount::new(RwLock::new(chunk)),
            chunk_cache: RefCount::default(),
            atom_interner,
            entrypoint,
        })
    }

    pub(super) fn read_instruction(&self, offset: Offset) -> Instruction {
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

    pub(super) fn procedure(&self, label: &str) -> Result<Value, ChunkError> {
        self.chunk
            .read()
            .unwrap()
            .labels
            .get(label)
            .map(|ip| Value::from(Procedure::new(*ip)))
            .ok_or_else(|| ChunkError::MissingLabel(label.to_owned()))
    }

    pub(super) fn annotations(&self, ip: Offset) -> Vec<Annotation> {
        self.chunk.read().unwrap().get_annotations(ip)
    }
}
