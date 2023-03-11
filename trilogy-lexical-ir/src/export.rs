use source_span::Span;

#[derive(Clone, Debug)]
pub struct Export {
    pub span: Span,
    pub name: String,
}
