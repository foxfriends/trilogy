use super::Error;
use super::{error::ErrorKind, loader::Loader};
use crate::location::Location;
use crate::Cache;
use ariadne::{ColorGenerator, Fmt, FnCache, Label, ReportKind};
use source_span::Span;
use std::fmt::{self, Debug};
use std::ops::Range;
use trilogy_parser::Spanned;

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

trait CacheExt<'a>: ariadne::Cache<&'a Location> {
    fn span(&mut self, location: &'a Location, span: Span) -> (&'a Location, Range<usize>) {
        match self.fetch(&location) {
            Ok(source) => {
                let start = source.line(span.start().line).unwrap().offset() + span.start().column;
                let end = source.line(span.end().line).unwrap().offset() + span.end().column;
                (location, start..end)
            }
            Err(..) => (location, 0..0),
        }
    }
}

impl<'a, C> CacheExt<'a> for C where C: ariadne::Cache<&'a Location> {}

impl<E: std::error::Error + 'static> Report<E> {
    pub fn eprint(&self) {
        let loader = Loader::new(self.cache.as_ref());
        let mut cache = FnCache::new(move |loc: &&Location| {
            loader
                .load_source(*loc)
                .map(|s| s.unwrap())
                .map_err(|e| Box::new(e) as Box<dyn Debug>)
        });

        let mut colors = ColorGenerator::from_state([30000, 15000, 35000], 0.35);
        let primary = colors.next();

        for error in &self.errors {
            let report = match &error.0 {
                ErrorKind::External(error) => {
                    eprintln!("{}", error);
                    continue;
                }
                ErrorKind::Analyzer(location, error) => {
                    use trilogy_ir::Error::*;
                    match error {
                        UnknownExport { name } => {
                            let span = cache.span(location, name.span());
                            ariadne::Report::build(ReportKind::Error, location, span.1.start)
                                .with_message(format!(
                                    "Exporting undeclared identifier `{}`",
                                    name.as_ref().fg(primary)
                                ))
                                .with_label(
                                    Label::new(span)
                                        .with_color(primary)
                                        .with_message("listed here"),
                                )
                        }
                        UnboundIdentifier { name } => {
                            let span = cache.span(location, name.span());
                            ariadne::Report::build(ReportKind::Error, location, span.1.start)
                                .with_message(format!(
                                    "Reference to undeclared identifier `{}`",
                                    name.as_ref().fg(primary),
                                ))
                                .with_label(
                                    Label::new(span)
                                        .with_color(primary)
                                        .with_message("referenced here"),
                                )
                        }
                        UnknownModule { name } => {
                            ariadne::Report::build(ReportKind::Error, location, 0)
                        }
                        DuplicateDefinition { name } => {
                            ariadne::Report::build(ReportKind::Error, location, 0)
                        }
                        IdentifierInOwnDefinition { name } => {
                            ariadne::Report::build(ReportKind::Error, location, 0)
                        }
                        DisjointBindings { span } => {
                            ariadne::Report::build(ReportKind::Error, location, 0)
                        }
                        AssignedImmutableBinding { name } => {
                            ariadne::Report::build(ReportKind::Error, location, 0)
                        }
                    }
                }
                ErrorKind::Syntax(location, error) => {
                    ariadne::Report::build(ReportKind::Error, location, 0)
                }
                ErrorKind::Resolver(location, error) => {
                    ariadne::Report::build(ReportKind::Error, location, 0)
                }
            };
            report.finish().eprint(&mut cache).unwrap();
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
