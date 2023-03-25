use source_span::Span;

#[derive(Clone, Debug)]
pub struct Identifier {
    span: Span,
    pub id: Id,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Id(usize);
