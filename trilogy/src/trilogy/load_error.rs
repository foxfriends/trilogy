use crate::location::Location;

use super::builder::ResolverError;
use std::fmt::{self, Display};
use trilogy_ir::Error as IrError;
use trilogy_parser::{syntax::SyntaxError, Spanned};

#[derive(Debug)]
pub struct LoadError<E: std::error::Error> {
    errors: Vec<ErrorKind<E>>,
}

impl<E: std::error::Error> LoadError<E> {
    pub(super) fn new(errors: Vec<ErrorKind<E>>) -> Self {
        Self { errors }
    }

    pub(super) fn new_empty() -> Self {
        Self::new(vec![])
    }

    pub(super) fn add<Er>(&mut self, error: Er)
    where
        ErrorKind<E>: From<Er>,
    {
        self.errors.push(error.into());
    }

    pub(super) fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

#[derive(Debug)]
pub(crate) enum ErrorKind<E: std::error::Error> {
    Resolver(ResolverError<E>),
    Syntax(Location, SyntaxError),
    Analyzer(Location, IrError),
}

impl<E: std::error::Error> std::error::Error for LoadError<E> {}

impl<E: std::error::Error> Display for LoadError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in &self.errors {
            match error {
                ErrorKind::Resolver(error) => {
                    writeln!(f, "{error:?}")?;
                }
                ErrorKind::Syntax(location, error) => {
                    let span = error.span();
                    let message = error.message();
                    writeln!(f, "{location}({span}): {message}")?;
                }
                ErrorKind::Analyzer(location, error) => {
                    writeln!(f, "{location}: {error:?}")?;
                }
            }
        }
        Ok(())
    }
}

impl<E: std::error::Error> From<ResolverError<E>> for ErrorKind<E> {
    fn from(value: ResolverError<E>) -> Self {
        Self::Resolver(value)
    }
}
