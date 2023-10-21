use super::loader::ResolverError;
use crate::location::Location;
use std::fmt::{self, Display};
use trilogy_ir::Error as IrError;
use trilogy_parser::{syntax::SyntaxError, Spanned};

#[derive(Debug)]
pub struct Error<E: std::error::Error>(pub(super) ErrorKind<E>);

#[derive(Debug)]
pub(super) enum ErrorKind<E: std::error::Error> {
    External(Box<dyn std::error::Error>),
    Resolver(Location, ResolverError<E>),
    Syntax(Location, SyntaxError),
    Analyzer(Location, IrError),
}

impl<E: std::error::Error> Error<E> {
    pub(super) fn external(e: impl std::error::Error + 'static) -> Self {
        Self(ErrorKind::External(Box::new(e)))
    }

    pub(super) fn resolution(location: Location, error: ResolverError<E>) -> Self {
        Self(ErrorKind::Resolver(location, error))
    }

    pub(super) fn syntax(location: Location, error: SyntaxError) -> Self {
        Self(ErrorKind::Syntax(location, error))
    }

    pub(super) fn semantic(location: Location, error: IrError) -> Self {
        Self(ErrorKind::Analyzer(location, error))
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
                writeln!(f, "{location} {error:?}")?;
            }
            ErrorKind::Syntax(location, error) => {
                let span = error.span();
                let message = error.message();
                writeln!(f, "{location} ({span}): {message}")?;
            }
            ErrorKind::Analyzer(location, error) => {
                writeln!(f, "{location}: {error:?}")?;
            }
        }
        Ok(())
    }
}
