use crate::Codegen;
use inkwell::values::PointerValue;
use trilogy_ir::ir;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn compile_iterator(
        &self,
        query: &ir::Query,
        done_to: PointerValue<'ctx>,
    ) -> Option<PointerValue<'ctx>> {
        let next_function = self.add_continuation("next");
        let brancher = self.end_continuation_point_as_branch();

        // TODO: declare variables here?
        let next_to =
            self.capture_current_continuation(next_function, &brancher, "next_continuation");
        let next_continuation_cp = self.hold_continuation_point();

        self.compile_query(&query.value, next_to, done_to)?;

        self.become_continuation_point(next_continuation_cp);
        self.begin_next_function(next_function);
        Some(self.get_continuation("next_iteration"))
    }

    fn compile_query(
        &self,
        query: &ir::QueryValue,
        next_to: PointerValue<'ctx>,
        done_to: PointerValue<'ctx>,
    ) -> Option<()> {
        match query {
            ir::QueryValue::Pass => {
                let next_iteration = self.add_continuation("pass_next");
                let brancher = self.end_continuation_point_as_branch();
                let next_iteration_continuation =
                    self.capture_current_continuation(next_iteration, &brancher, "pass_next");
                let next_iteration_cp = self.hold_continuation_point();
                self.call_known_continuation(next_to, next_iteration_continuation);

                self.become_continuation_point(next_iteration_cp);
                self.begin_next_function(next_iteration);
                let done_to = self.use_temporary(done_to).unwrap();
                self.void_call_continuation(done_to);
            }
            ir::QueryValue::End => {
                let done_to = self.use_temporary(done_to).unwrap();
                self.void_call_continuation(done_to);
            }
            _ => todo!(),
        }
        Some(())
    }
}
