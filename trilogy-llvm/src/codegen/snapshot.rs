use super::Codegen;
use super::debug_info::DebugScope;
use inkwell::debug_info::DILocation;
use inkwell::values::PointerValue;

#[derive(Clone, Debug)]
pub(crate) struct Snapshot<'ctx> {
    closure_array: Option<PointerValue<'ctx>>,
    params: Vec<PointerValue<'ctx>>,
    debug_stack: Vec<DebugScope<'ctx>>,
    debug_location: DILocation<'ctx>,
}

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn snapshot_function_context(&self) -> Snapshot<'ctx> {
        Snapshot {
            closure_array: self.closure_array.get(),
            params: self.function_params.borrow().clone(),
            debug_stack: self.di.debug_scopes.borrow().clone(),
            debug_location: self.builder.get_current_debug_location().unwrap(),
        }
    }

    pub(crate) fn restore_function_context(&self, snapshot: Snapshot<'ctx>) {
        self.closure_array.set(snapshot.closure_array);
        *self.function_params.borrow_mut() = snapshot.params;
        *self.di.debug_scopes.borrow_mut() = snapshot.debug_stack;
        self.builder
            .set_current_debug_location(snapshot.debug_location);
    }
}
