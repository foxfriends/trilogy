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
