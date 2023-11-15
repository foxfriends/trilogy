use trilogy_vm::Offset;

pub(crate) trait StackTracker {
    fn intermediate(&mut self) -> Offset;
    fn end_intermediate(&mut self) -> &mut Self;
}
