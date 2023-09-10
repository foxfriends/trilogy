use std::sync::Arc;
use trilogy_ir::ir;
use trilogy_vm as vm;
use vm::OpCode;

#[derive(Debug)]
pub struct Program {
    module: Arc<ir::ModuleCell>,
}

impl Program {
    pub(crate) fn new(module: Arc<ir::ModuleCell>) -> Self {
        Self { module }
    }

    pub fn generate_code(self) -> vm::Program {
        let mut builder = vm::ProgramBuilder::default();
        builder.write_opcode(OpCode::Jump);
        builder.write_offset_label("main".to_owned());
        trilogy_codegen::write_program(&mut builder, self.module.as_module().unwrap());
        builder.build().unwrap()
    }
}
