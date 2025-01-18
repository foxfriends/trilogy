use inkwell::{
    basic_block::BasicBlock,
    values::{FunctionValue, PointerValue},
};
use std::collections::HashMap;
use trilogy_ir::Id;

pub(crate) struct Scope<'ctx> {
    pub(crate) function: FunctionValue<'ctx>,
    pub(crate) variables: HashMap<Id, PointerValue<'ctx>>,
    pub(crate) cleanup: Option<BasicBlock<'ctx>>,
}

impl<'ctx> Scope<'ctx> {
    pub(crate) fn begin(function: FunctionValue<'ctx>) -> Scope<'ctx> {
        Scope {
            function,
            variables: HashMap::default(),
            cleanup: None,
        }
    }

    pub(crate) fn set_cleanup(&mut self, cleanup: BasicBlock<'ctx>) {
        self.cleanup = Some(cleanup);
    }

    pub(crate) fn sret(&self) -> PointerValue<'ctx> {
        self.function.get_nth_param(0).unwrap().into_pointer_value()
    }
}
