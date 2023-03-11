use source_span::Span;

#[derive(Clone, Debug)]
pub enum LexicalError {
    ExportedMultipleTimes {
        original: Span,
        duplicate: Span,
        name: String,
    },
    ConflictingDefinition {
        name: String,
        original: Span,
        conflict: Span,
    },
}
