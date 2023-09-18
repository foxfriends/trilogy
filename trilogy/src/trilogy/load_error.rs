use super::builder::ResolverError;
use std::fmt::{self, Display};
use trilogy_ir::Error as IrError;
use trilogy_parser::syntax::SyntaxError;

#[derive(Debug)]
pub enum LoadError<E: std::error::Error> {
    Resolver(Vec<ResolverError<E>>),
    Syntax(Vec<SyntaxError>),
    Analyzer(Vec<IrError>),
}

impl<E: std::error::Error> std::error::Error for LoadError<E> {}

impl<E: std::error::Error> Display for LoadError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resolver(errors) => {
                for error in errors {
                    writeln!(f, "{error:?}")?;
                }
                Ok(())
            }
            Self::Syntax(errors) => {
                for error in errors {
                    writeln!(f, "{error:?}")?;
                }
                Ok(())
            }
            Self::Analyzer(errors) => {
                for error in errors {
                    writeln!(f, "{error:?}")?;
                }
                Ok(())
            }
        }
    }
}
