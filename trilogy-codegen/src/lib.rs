use trilogy_ir::ir;
use trilogy_vm::{Instruction, ProgramBuilder};

pub fn write_module(builder: &mut ProgramBuilder, module: &ir::Module) {
    builder
        .write_label(module.location().to_owned())
        .expect("each module has a unique location and is only written once")
        .write_instruction(Instruction::Exit);
}
