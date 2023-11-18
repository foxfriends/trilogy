use trilogy_vm::Offset;

pub(crate) trait StackTracker {
    fn intermediate(&mut self) -> Offset;
    fn end_intermediate(&mut self) -> &mut Self;

    fn push_continue(&mut self, offset: Offset) -> &mut Self;
    fn pop_continue(&mut self) -> &mut Self;

    fn push_break(&mut self, offset: Offset) -> &mut Self;
    fn pop_break(&mut self) -> &mut Self;

    fn push_resume(&mut self, offset: Offset) -> &mut Self;
    fn pop_resume(&mut self) -> &mut Self;

    fn push_cancel(&mut self, offset: Offset) -> &mut Self;
    fn pop_cancel(&mut self) -> &mut Self;
}

#[macro_export]
macro_rules! delegate_stack_tracker {
    ($t:ty, $f:ident) => {
        impl StackTracker for $t {
            fn intermediate(&mut self) -> trilogy_vm::Offset {
                self.$f.intermediate()
            }

            fn end_intermediate(&mut self) -> &mut Self {
                self.$f.end_intermediate();
                self
            }

            fn push_continue(&mut self, offset: trilogy_vm::Offset) -> &mut Self {
                self.$f.push_continue(offset);
                self
            }

            fn pop_continue(&mut self) -> &mut Self {
                self.$f.pop_continue();
                self
            }

            fn push_break(&mut self, offset: trilogy_vm::Offset) -> &mut Self {
                self.$f.push_break(offset);
                self
            }

            fn pop_break(&mut self) -> &mut Self {
                self.$f.pop_break();
                self
            }

            fn push_cancel(&mut self, offset: trilogy_vm::Offset) -> &mut Self {
                self.$f.push_cancel(offset);
                self
            }

            fn pop_cancel(&mut self) -> &mut Self {
                self.$f.pop_cancel();
                self
            }

            fn push_resume(&mut self, offset: trilogy_vm::Offset) -> &mut Self {
                self.$f.push_resume(offset);
                self
            }

            fn pop_resume(&mut self) -> &mut Self {
                self.$f.pop_resume();
                self
            }
        }
    };
}
