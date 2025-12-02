use crate::Codegen;
use inkwell::values::FunctionValue;
use trilogy_ir::ir;

impl<'ctx> Codegen<'ctx> {
    fn write_test_accessor(&self, accessor: FunctionValue<'ctx>, accessing: FunctionValue<'ctx>) {
        let accessor_entry = self.context.append_basic_block(accessor, "entry");
        self.builder.position_at_end(accessor_entry);
        let sret = accessor.get_nth_param(0).unwrap().into_pointer_value();
        self.trilogy_callable_init_proc(sret, 0, accessing);
        self.builder.build_return(None).unwrap();
    }

    pub(crate) fn compile_test(&self, test: &ir::TestDefinition) {
        let name = test.name.to_string();
        let linkage_name = format!("test#{name}");
        let accessor_name = format!("{}::{}", self.module_path(), name);
        let accessor = self.add_test(&accessor_name);
        let function = self.add_procedure(&linkage_name, 0, &name, test.span, false);
        self.write_test_accessor(accessor, function);
        self.set_current_definition(name.to_owned(), linkage_name.to_owned(), test.span, None);
        self.compile_test_body(function, test);
        self.close_continuation();
    }

    pub(crate) fn compile_test_body(
        &self,
        function: FunctionValue<'ctx>,
        test: &ir::TestDefinition,
    ) {
        self.begin_function(function, test.span);
        if let Some(value) = self.compile_expression(&test.body, "") {
            self.trilogy_value_destroy(value);
            let ret = self.get_return("");
            self.call_known_continuation(ret, self.allocate_const(self.unit_const(), ""));
        }
        self.end_function();
    }
}
