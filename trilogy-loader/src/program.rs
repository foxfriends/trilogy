use std::sync::Arc;
use trilogy_ir::ir;
use trilogy_vm as vm;

#[derive(Debug)]
pub struct Program {
    #[allow(dead_code)]
    module: Arc<ir::ModuleCell>,
}

impl Program {
    pub(crate) fn new(module: Arc<ir::ModuleCell>) -> Self {
        Self { module }
    }

    pub fn generate_code(self) -> vm::Program {
        let mut builder = vm::ProgramBuilder::default();
        trilogy_codegen::write_module(&mut builder, self.module.as_module().unwrap());
        builder.build().unwrap()
    }
}
