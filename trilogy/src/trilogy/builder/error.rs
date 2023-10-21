use super::loader::ResolverError;
use crate::location::Location;
use std::fmt::{self, Display};
use trilogy_ir::Error as IrError;
use trilogy_parser::{syntax::SyntaxError, Spanned};

#[derive(Debug)]
pub struct Error<E: std::error::Error>(ErrorKind<E>);

#[derive(Debug)]
enum ErrorKind<E: std::error::Error> {
    Resolver(ResolverError<E>),
    Syntax(Location, SyntaxError),
    Analyzer(Location, IrError),
}

impl<E: std::error::Error> Error<E> {
    pub(super) fn resolution(error: ResolverError<E>) -> Self {
        Self(ErrorKind::Resolver(error))
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
            ErrorKind::Resolver(error) => {
                writeln!(f, "{error:?}")?;
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
