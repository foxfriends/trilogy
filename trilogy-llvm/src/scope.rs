use inkwell::values::{FunctionValue, PointerValue};
use std::collections::HashMap;
use trilogy_ir::Id;

pub(crate) struct Scope<'ctx> {
    pub(crate) function: FunctionValue<'ctx>,
    pub(crate) variables: HashMap<Id, PointerValue<'ctx>>,
}

impl<'ctx> Scope<'ctx> {
    pub(crate) fn begin(function: FunctionValue<'ctx>) -> Scope<'ctx> {
        Scope {
            function,
            variables: HashMap::default(),
        }
    }

    pub(crate) fn sret(&self) -> PointerValue<'ctx> {
        self.function.get_nth_param(0).unwrap().into_pointer_value()
    }
}
