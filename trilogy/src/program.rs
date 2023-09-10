use trilogy_ir::ir;
use trilogy_vm as vm;
use vm::OpCode;

#[derive(Debug)]
pub struct Program {
    module: ir::Module,
}

impl Program {
    pub(crate) fn new(module: ir::Module) -> Self {
        Self { module }
    }

    pub fn generate_code(self) -> vm::Program {
        let mut builder = vm::ProgramBuilder::default();
        builder.write_opcode(OpCode::Jump);
        builder.write_offset_label("main".to_owned());
        trilogy_codegen::write_program(&mut builder, &self.module);
        builder.build().unwrap()
    }
}
