#[derive(Clone, Debug)]
pub(crate) enum StaticMember {
    Chunk(String),
    Context(String),
    Label(String),
}

impl StaticMember {
    pub fn unwrap_label(self) -> String {
        match self {
            Self::Label(label) => label,
            _ => panic!("expected static member to be a label, but it was {self:?}"),
        }
    }

    pub fn unwrap_context(self) -> String {
        match self {
            Self::Context(label) => label,
            _ => panic!("expected static member to be in context, but it was {self:?}"),
        }
    }
}
