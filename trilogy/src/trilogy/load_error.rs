use std::fmt::{self, Display};
use trilogy_ir::Error as IrError;
use trilogy_parser::syntax::SyntaxError;

#[derive(Debug)]
pub enum LoadError<E: std::error::Error> {
    InvalidScheme(String),
    Cache(E),
    Syntax(Vec<SyntaxError>),
    Analyzer(Vec<IrError>),
    External(Box<dyn std::error::Error>),
}

impl<E: std::error::Error> LoadError<E> {
    pub(crate) fn external(error: impl std::error::Error + 'static) -> Self {
        Self::External(Box::new(error))
    }
}

impl<E: std::error::Error> std::error::Error for LoadError<E> {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Self::External(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

impl<E: std::error::Error> Display for LoadError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cache(error) => write!(f, "{error}"),
            Self::InvalidScheme(scheme) => {
                write!(f, "invalid scheme in module location `{}`", scheme)
            }
            Self::Syntax(errors) => {
                for error in errors {
                    writeln!(f, "{error:#?}")?;
                }
                Ok(())
            }
            Self::Analyzer(errors) => {
                for error in errors {
                    writeln!(f, "{error:#?}")?;
                }
                Ok(())
            }
            Self::External(error) => write!(f, "{error}"),
        }
    }
}
