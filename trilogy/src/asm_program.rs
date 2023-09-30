use crate::{location::Location, NativeModule};
use std::io::Read;
use std::path::Path;
use std::{collections::HashMap, io};
use trilogy_vm::{ChunkBuilder, Program, Value};

pub struct AsmProgram {
    source: String,
    libraries: HashMap<Location, NativeModule>,
}

impl AsmProgram {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        Self::read(&mut std::fs::File::open(path)?)
    }

    pub fn read<R: Read>(source: &mut R) -> Result<Self, std::io::Error> {
        let source = io::read_to_string(source)?;
        Ok(Self {
            source,
            libraries: HashMap::default(),
        })
    }
}

impl Program for AsmProgram {
    fn entrypoint(&self, chunk: &mut ChunkBuilder) {
        chunk
            .parse(&self.source)
            .entrypoint_existing("trilogy:__entrypoint__");
    }

    fn chunk(&self, locator: &Value, chunk: &mut ChunkBuilder) {
        let location = match locator {
            Value::String(url) => Location::absolute(url.parse().expect("invalid module location")),
            _ => panic!("invalid module specifier `{locator}`"),
        };
        if let Some(_lib) = self.libraries.get(&location) {
            todo!("native modules")
        } else {
            chunk.entrypoint_existing(&format!("location:{}", location));
        }
    }
}
