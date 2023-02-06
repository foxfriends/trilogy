use super::*;

#[derive(Clone, Debug, Spanned)]
pub struct MemberAccess {
    pub path: Path,
    // Identifiers at the beginning of `segments` are fluid,
    // they might actually belong at the end of `path.
    pub segments: Vec<Member>,
}

#[derive(Clone, Debug, Spanned)]
pub enum Member {
    Static(Box<Identifier>),
    Dynamic(Box<Expression>),
}
