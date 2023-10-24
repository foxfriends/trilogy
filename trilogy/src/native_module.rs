use crate::location::Location;
use std::collections::HashMap;
use trilogy_vm::{ChunkBuilder, Instruction, Native, NativeFunction};

#[derive(Clone, Debug)]
pub struct NativeModule {
    pub(crate) modules: HashMap<&'static str, NativeModule>,
    pub(crate) procedures: HashMap<&'static str, Native>,
}

#[derive(Clone)]
pub struct NativeModuleBuilder {
    inner: NativeModule,
}

impl Default for NativeModuleBuilder {
    fn default() -> Self {
        Self {
            inner: NativeModule {
                modules: Default::default(),
                procedures: Default::default(),
            },
        }
    }
}

impl NativeModuleBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_submodule(mut self, name: &'static str, module: NativeModule) -> Self {
        self.inner.modules.insert(name, module);
        self
    }

    pub fn add_procedure<N: NativeFunction + 'static>(
        mut self,
        name: &'static str,
        proc: N,
    ) -> Self {
        self.inner.procedures.insert(name, proc.into());
        self
    }

    pub fn build(self) -> NativeModule {
        self.inner
    }
}

impl NativeModule {
    pub(crate) fn write_to_chunk(&self, location: &Location, chunk: &mut ChunkBuilder) {
        self.write_to_chunk_at_path(location, vec![], chunk)
    }

    fn write_to_chunk_at_path(
        &self,
        location: &Location,
        path: Vec<&str>,
        chunk: &mut ChunkBuilder,
    ) {
        let pathstr = path.iter().fold(String::new(), |s, seg| s + seg + "::");
        for (name, proc) in &self.procedures {
            let atom = chunk.atom(name);
            let next = format!("#skip::{location}::{pathstr}{name}");
            chunk
                .instruction(Instruction::Copy)
                .instruction(Instruction::Const(atom.into()))
                .instruction(Instruction::ValEq)
                .cond_jump(&next)
                .instruction(Instruction::Const(proc.clone().into()))
                .instruction(Instruction::Return)
                .label(next);
        }
        for (name, module) in &self.modules {
            let atom = chunk.atom(name);
            let next = format!("#skip::{location}::{pathstr}{name}");
            let module_label = format!("{location}::{pathstr}{name}");
            chunk
                .instruction(Instruction::Copy)
                .instruction(Instruction::Const(atom.into()))
                .instruction(Instruction::ValEq)
                .cond_jump(&next)
                .reference(&module_label)
                .instruction(Instruction::Return)
                .label(module_label);
            module.write_to_chunk(location, chunk);
            chunk.label(next);
        }
        chunk.instruction(Instruction::Fizzle);
    }
}
