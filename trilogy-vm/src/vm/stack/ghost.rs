use super::Stack;

#[derive(Clone, Debug)]
pub(crate) struct Ghost {
    pub stack: Stack,
    pub len: usize,
}

impl From<Stack> for Ghost {
    fn from(stack: Stack) -> Self {
        Self {
            len: stack.count_locals(),
            stack,
        }
    }
}
