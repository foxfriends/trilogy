use inkwell::{
    debug_info::{DIBasicType, DISubroutineType},
    llvm_sys::debuginfo::LLVMDIFlagPublic,
};

use crate::codegen::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn value_di_type(&self) -> DIBasicType<'ctx> {
        self.dibuilder
            .create_basic_type("trilogy_value", 8 * 9, 0, LLVMDIFlagPublic)
            .unwrap()
    }

    pub(crate) fn procedure_di_type(&self, arity: usize) -> DISubroutineType<'ctx> {
        self.dibuilder.create_subroutine_type(
            self.dicu.get_file(),
            Some(self.value_di_type().as_type()),
            &vec![self.value_di_type().as_type(); arity],
            LLVMDIFlagPublic,
        )
    }
}
