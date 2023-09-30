use crate::{location::Location, NativeModule};
use std::collections::HashMap;
use trilogy_ir::ir::Module;
use trilogy_vm::{ChunkBuilder, Program, Value};

pub(crate) struct TrilogyProgram<'a> {
    pub modules: &'a HashMap<Location, Module>,
    pub libraries: &'a HashMap<Location, NativeModule>,
    pub entrypoint: &'a Location,
}

impl Program for TrilogyProgram<'_> {
    fn entrypoint(&self, chunk: &mut ChunkBuilder) {
        let module = self.modules.get(self.entrypoint).unwrap();
        trilogy_codegen::write_program(chunk, module);
    }

    fn chunk(&self, locator: &Value, chunk: &mut ChunkBuilder) {
        let location = match locator {
            Value::String(url) => Location::absolute(url.parse().expect("invalid module location")),
            _ => panic!("invalid module specifier `{locator}`"),
        };
        enum Either<'a> {
            Source(&'a Module),
            Native(&'a NativeModule),
        }
        let module = self
            .modules
            .get(&location)
            .map(Either::Source)
            .or_else(|| self.libraries.get(&location).map(Either::Native))
            .expect("unknown module location");
        chunk.label(format!("location:{location}"));
        match module {
            Either::Source(module) => trilogy_codegen::write_module(chunk, module),
            Either::Native(..) => todo!("native modules"),
        }
    }
}
