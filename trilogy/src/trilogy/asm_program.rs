use crate::{location::Location, NativeModule};
use std::collections::HashMap;
use trilogy_vm::{ChunkBuilder, Program, Value};

pub(super) struct AsmProgram<'a> {
    pub source: &'a str,
    pub libraries: &'a HashMap<Location, NativeModule>,
}

impl Program for AsmProgram<'_> {
    fn entrypoint(&self, chunk: &mut ChunkBuilder) {
        chunk
            .parse(self.source)
            .entrypoint_existing("trilogy:__entrypoint__");
    }

    fn chunk(&self, locator: &Value, chunk: &mut ChunkBuilder) {
        let location = match locator {
            Value::String(url) => Location::absolute(url.parse().expect("invalid module location")),
            _ => panic!("invalid module specifier `{locator}`"),
        };
        if let Some(lib) = self.libraries.get(&location) {
            lib.write_to_chunk(&location, chunk);
        } else {
            chunk.entrypoint_existing(&format!("location:{}", location));
        }
    }
}
