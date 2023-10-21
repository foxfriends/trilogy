use super::Error;
use super::{error::ErrorKind, loader::Loader};
use crate::location::Location;
use crate::Cache;
use ariadne::{FnCache, ReportKind};
use source_span::Span;
use std::fmt::{self, Debug, Display};

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

struct ErrorSpan<'a> {
    location: &'a Location,
    start: usize,
    end: usize,
    span: Span,
}

impl<'a> ariadne::Span for ErrorSpan<'a> {
    type SourceId = &'a Location;

    fn source(&self) -> &Self::SourceId {
        &self.location
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}

impl<E: std::error::Error + 'static> Report<E> {
    pub fn eprint(&self) {
        let loader = Loader::new(self.cache.as_ref());
        let mut cache = FnCache::new(move |loc: &&Location| {
            loader
                .load_source(*loc)
                .map(|s| s.unwrap())
                .map_err(|e| Box::new(e) as Box<dyn Debug>)
        });

        for error in &self.errors {
            let report = match &error.0 {
                ErrorKind::External(error) => {
                    eprintln!("{}", error);
                    continue;
                }
                ErrorKind::Analyzer(location, error) => {
                    ariadne::Report::<ErrorSpan>::build(ReportKind::Error, location, 0)
                }
                ErrorKind::Syntax(location, error) => {
                    ariadne::Report::build(ReportKind::Error, location, 0)
                }
                ErrorKind::Resolver(location, error) => {
                    ariadne::Report::build(ReportKind::Error, location, 0)
                }
            };
            report.finish().eprint(&mut cache);
        }
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
