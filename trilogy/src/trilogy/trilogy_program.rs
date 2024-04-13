use crate::location::Location;
use std::{collections::HashMap, time::Instant};
use trilogy_ir::ir::Module;
use trilogy_vm::{ChunkBuilder, ChunkWriter, Native, Program, Value};

pub(super) struct TrilogyProgram<'a> {
    pub modules: &'a HashMap<Location, Module>,
    pub libraries: &'a HashMap<Location, Native>,
    pub asm_modules: &'a HashMap<Location, String>,
    pub entrypoint: &'a Location,
    pub path: &'a [&'a str],
    pub parameters: Vec<Value>,
    pub to_asm: bool,
}

impl Program for TrilogyProgram<'_> {
    fn entrypoint(&self, chunk: &mut ChunkBuilder) {
        let time_generating = Instant::now();
        let module = self.modules.get(self.entrypoint).unwrap();
        let url = self.entrypoint.as_ref().as_str();
        trilogy_codegen::write_program(url, chunk, module, self.path, self.parameters.clone());
        log::trace!("entrypoint written: {:?}", time_generating.elapsed());
    }

    fn chunk(&self, locator: &Value, chunk: &mut ChunkBuilder) {
        let time_generating = Instant::now();
        let location = match locator {
            Value::String(url) => Location::absolute(url.parse().expect("invalid module location")),
            _ => panic!("invalid module specifier `{locator}`"),
        };
        log::debug!("loading chunk: {}", location);
        enum Either<'a> {
            Source(&'a Module),
            Native(&'a Native),
            Asm(&'a str),
        }
        let module = self
            .modules
            .get(&location)
            .map(Either::Source)
            .or_else(|| self.libraries.get(&location).map(Either::Native))
            .or_else(|| self.asm_modules.get(&location).map(|s| Either::Asm(s)))
            .unwrap_or_else(|| panic!("unknown module location `{location}`"));
        let url = location.as_ref().as_str();
        match module {
            Either::Source(module) => {
                chunk.label(format!("location:{location}"));
                trilogy_codegen::write_module(url, chunk, module)
            }
            Either::Native(module) => {
                if self.to_asm {
                    return;
                }
                chunk
                    .label(format!("location:{location}"))
                    .constant(module.clone())
                    .instruction(trilogy_vm::Instruction::Return);
            }
            Either::Asm(raw) => {
                chunk.label(format!("location:{location}")).parse(raw);
            }
        }
        log::trace!("chunk written: {:?}", time_generating.elapsed());
    }
}
