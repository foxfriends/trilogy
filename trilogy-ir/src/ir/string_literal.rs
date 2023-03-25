use source_span::Span;

#[derive(Clone, Debug)]
pub struct StringLiteral {
    span: Span,
    pub value: String,
}
