use std::fmt::{self, Debug};

use super::Error;
use crate::Cache;

pub struct Report<E: std::error::Error> {
    cache: Box<dyn Cache<Error = E>>,
    errors: Vec<Error<E>>,
    warnings: Vec<Error<E>>,
}

impl<E: std::error::Error> Debug for Report<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Report")
            .field("errors", &self.errors)
            .field("warnings", &self.warnings)
            .finish_non_exhaustive()
    }
}

impl<E: std::error::Error> Report<E> {
    pub fn display(self) -> Result<impl std::fmt::Display, std::io::Error> {
        Ok("You have errors")
    }
}

pub(super) struct ReportBuilder<E: std::error::Error> {
    errors: Vec<Error<E>>,
    warnings: Vec<Error<E>>,
}

impl<E: std::error::Error> Default for ReportBuilder<E> {
    fn default() -> Self {
        Self {
            errors: vec![],
            warnings: vec![],
        }
    }
}

impl<E: std::error::Error> ReportBuilder<E> {
    pub fn error(&mut self, error: Error<E>) {
        self.errors.push(error);
    }

    pub fn warning(&mut self, warning: Error<E>) {
        self.warnings.push(warning);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn report<C: Cache<Error = E> + 'static>(self, cache: C) -> Report<E> {
        Report {
            cache: Box::new(cache),
            errors: self.errors,
            warnings: self.warnings,
        }
    }

    pub fn checkpoint<C: Cache<Error = E> + 'static>(&mut self, cache: C) -> Result<C, Report<E>> {
        if self.has_errors() {
            Err(std::mem::take(self).report(cache))
        } else {
            Ok(cache)
        }
    }
}
