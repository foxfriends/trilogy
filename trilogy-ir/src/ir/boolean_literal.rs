use source_span::Span;

#[derive(Clone, Debug)]
pub struct BooleanLiteral {
    span: Span,
    pub value: bool,
}
