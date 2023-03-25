use source_span::Span;

#[derive(Clone, Debug)]
pub struct CharacterLiteral {
    span: Span,
    pub value: char,
}
