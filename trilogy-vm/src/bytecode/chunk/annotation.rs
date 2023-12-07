use crate::Offset;
use source_span::Span;

/// The fully specified location of a fragment of the source program.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Location {
    /// The path to the source file.
    pub file: String,
    /// The position within the source file.
    pub span: Span,
}

/// Marks an annotated range as having some meaning.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Note {
    /// The annotated range corresponds to a span in the source code.
    Source {
        /// A name for this range, if applicable, recognizable to the developer.
        ///
        /// Likely a function name or similar.
        name: Option<String>,
        /// The source location.
        location: Location,
    },
    /// Within the annotated range, this memory offset corresponds to a particular value.
    Memory {
        /// The name of this memory offset, recognizable to the developer.
        ///
        /// Likely a variable name or similar.
        name: String,
        /// The offset on the stack that is being named.
        offset: Offset,
    },
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Annotation {
    /// The first instruction from which this annotation is in effect.
    pub start: Offset,
    /// The last in struction for which this annotation is in effect.
    pub end: Offset,
    /// The value of this annotation.
    pub note: Note,
}
