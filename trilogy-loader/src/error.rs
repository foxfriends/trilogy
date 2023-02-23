use std::fmt::{self, Display};

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ErrorKind {
    Inaccessible,
    Invalid,
    InvalidLocation,
    Cache,
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    source: Option<Box<dyn std::error::Error>>,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind, source: impl std::error::Error + 'static) -> Self {
        Self {
            kind,
            source: Some(Box::new(source)),
        }
    }

    pub(crate) fn cache<E: std::error::Error + 'static>(error: E) -> Self {
        Self::new(ErrorKind::Cache, error)
    }

    pub(crate) fn inaccessible<E: std::error::Error + 'static>(error: E) -> Self {
        Self::new(ErrorKind::Inaccessible, error)
    }

    pub(crate) fn invalid<E: std::error::Error + 'static>(error: E) -> Self {
        Self::new(ErrorKind::Invalid, error)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.source {
            Some(source) => write!(f, "Loader Error ({:?}): {}", self.kind, source),
            None => write!(f, "Loader Error ({:?}): unknown error", self.kind),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self { kind, source: None }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_deref()
    }
}
