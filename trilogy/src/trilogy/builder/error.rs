use super::loader;
use crate::location::Location;
use std::fmt::{self, Display};
use trilogy_parser::syntax::SyntaxError;

#[derive(Debug)]
pub struct Error<E: std::error::Error>(pub(super) ErrorKind<E>);

#[derive(Debug)]
pub(super) enum ErrorKind<E: std::error::Error> {
    External(Box<dyn std::error::Error>),
    Resolver(Location, loader::Error<E>),
    Syntax(Location, SyntaxError),
    Ir(Location, trilogy_ir::Error),
    Analysis(Location, super::analyzer::ErrorKind),
}

impl<E: std::error::Error> Error<E> {
    pub(super) fn external(e: impl std::error::Error + 'static) -> Self {
        Self(ErrorKind::External(Box::new(e)))
    }

    pub(super) fn resolution(location: Location, error: loader::Error<E>) -> Self {
        Self(ErrorKind::Resolver(location, error))
    }

    pub(super) fn syntax(location: Location, error: SyntaxError) -> Self {
        Self(ErrorKind::Syntax(location, error))
    }

    pub(super) fn ir(location: Location, error: trilogy_ir::Error) -> Self {
        Self(ErrorKind::Ir(location, error))
    }

    pub(super) fn analysis(location: Location, error: super::analyzer::ErrorKind) -> Self {
        Self(ErrorKind::Analysis(location, error))
    }
}

impl<E: std::error::Error> From<Box<dyn std::error::Error>> for Error<E> {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Self(ErrorKind::External(value))
    }
}

impl<E: std::error::Error> std::error::Error for Error<E> {}

impl<E: std::error::Error> Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            ErrorKind::External(error) => {
                writeln!(f, "{error}")?;
            }
            ErrorKind::Resolver(location, error) => {
                writeln!(f, "{location} {error}")?;
            }
            ErrorKind::Syntax(location, error) => {
                writeln!(f, "{location} {error}")?;
            }
            ErrorKind::Ir(location, error) => {
                writeln!(f, "{location}: {error}")?;
            }
            ErrorKind::Analysis(location, error) => {
                writeln!(f, "{location}: {error}")?;
            }
        }
        Ok(())
    }
}
