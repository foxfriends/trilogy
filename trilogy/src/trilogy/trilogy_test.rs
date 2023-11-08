use crate::{location::Location, NativeModule};
use std::collections::HashMap;
use trilogy_ir::ir::Module;
use trilogy_vm::{ChunkBuilder, Program, Value};

pub(super) struct TrilogyTest<'a> {
    pub modules: &'a HashMap<Location, Module>,
    pub libraries: &'a HashMap<Location, NativeModule>,
    pub entrypoint: &'a Location,
    pub path: &'a [&'a str],
    pub test: &'a str,
    pub to_asm: bool,
}

impl Program for TrilogyTest<'_> {
    fn entrypoint(&self, chunk: &mut ChunkBuilder) {
        let module = self.modules.get(self.entrypoint).unwrap();
        trilogy_codegen::write_test(chunk, module, self.path, self.test);
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
            .unwrap_or_else(|| panic!("unknown module location `{location}`"));
        match module {
            Either::Source(module) => {
                chunk.label(format!("location:{location}"));
                trilogy_codegen::write_module(chunk, module)
            }
            Either::Native(module) => {
                if self.to_asm {
                    return;
                }
                chunk.label(format!("location:{location}"));
                module.write_to_chunk(&location, chunk);
            }
        }
    }
}