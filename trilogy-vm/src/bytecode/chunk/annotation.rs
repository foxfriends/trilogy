use crate::Offset;
use source_span::Span;
use std::cmp::Ordering;
use std::fmt::{self, Display};

/// The fully specified location of a fragment of the source program.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Location {
    /// The path to the source file.
    pub file: String,
    /// The position within the source file.
    pub span: Span,
}

impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.file != other.file {
            return None;
        }
        if self.span == other.span {
            return Some(Ordering::Equal);
        }
        if self.span.includes(&other.span) {
            return Some(Ordering::Greater);
        }
        if other.span.includes(&self.span) {
            return Some(Ordering::Less);
        }
        None
    }
}

impl Location {
    pub fn new<S: Into<String>>(file: S, span: Span) -> Self {
        Self {
            file: file.into(),
            span,
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.file, self.span)
    }
}

/// Marks an annotated range as having some meaning.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Note {
    /// The annotated range corresponds to a span in the source code.
    Source {
        /// A name for this range, if applicable, recognizable to the developer.
        ///
        /// Likely a function name or similar.
        name: String,
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
    /// A generic note
    Note { note: String },
}

impl Note {
    pub fn source(name: String, location: Location) -> Self {
        Self::Source { name, location }
    }

    pub(crate) fn into_source(self) -> Option<(String, Location)> {
        match self {
            Self::Source { name, location } => Some((name, location)),
            _ => None,
        }
    }

    pub(crate) fn into_note(self) -> Option<String> {
        match self {
            Self::Note { note } => Some(note),
            _ => None,
        }
    }
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

impl Annotation {
    pub fn source(start: Offset, end: Offset, name: String, location: Location) -> Self {
        Self {
            start,
            end,
            note: Note::Source { name, location },
        }
    }

    pub fn note(start: Offset, end: Offset, note: String) -> Self {
        Self {
            start,
            end,
            note: Note::Note { note },
        }
    }

    pub fn spans(&self, offset: Offset) -> bool {
        self.start <= offset && offset < self.end
    }
}
