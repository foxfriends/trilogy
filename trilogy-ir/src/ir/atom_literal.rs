use source_span::Span;

#[derive(Clone, Debug)]
pub struct AtomLiteral {
    span: Span,
    pub value: Atom,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Atom(String);
