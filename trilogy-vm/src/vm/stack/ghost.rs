use super::Stack;

#[derive(Clone, Debug)]
pub(crate) struct Ghost {
    pub stack: Stack,
}

impl From<Stack> for Ghost {
    fn from(stack: Stack) -> Self {
        Self { stack }
    }
}
