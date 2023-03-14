use source_span::Span;

#[derive(Clone, Debug)]
pub struct Export {
    pub span: Span,
    pub name: String,
}

impl Export {
    pub fn new(span: Span, name: String) -> Self {
        Self { span, name }
    }
}
