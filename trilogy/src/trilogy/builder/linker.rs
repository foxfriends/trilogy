use crate::{location::Location, LoadError, NativeModule, Program};
use std::collections::HashMap;
use trilogy_ir::ir;

#[allow(dead_code)]
struct Linker {
    libraries: HashMap<Location, NativeModule>,
    ir: HashMap<Location, ir::Module>,
}

pub fn link<E: std::error::Error>(
    libraries: HashMap<Location, NativeModule>,
    mut ir: HashMap<Location, ir::Module>,
    entrypoint: &Location,
) -> Result<Program, LoadError<E>> {
    let module = ir.remove(entrypoint).unwrap();
    let _linker = Linker { libraries, ir };
    Ok(Program::new(module))
}
