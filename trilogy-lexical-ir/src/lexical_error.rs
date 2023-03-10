use source_span::Span;

#[derive(Clone, Debug)]
pub struct LexicalError {
    span: Span,
    message: String,
}
