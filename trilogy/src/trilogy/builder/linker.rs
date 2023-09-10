use trilogy_ir::ir::Module;

use crate::NativeModule;

pub fn link(
    libraries: HashMap<Location, NativeModule>,
    ir: HashMap<Location, Module>,
    entrypoint: Location,
) -> crate::Program {
    todo!()
}
