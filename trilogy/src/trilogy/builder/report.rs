use super::error::ErrorKind;
use super::loader::Loader;
use super::Error;
use crate::location::Location;
use crate::Cache;
use ariadne::{ColorGenerator, Config, Fmt, FnCache, Label, ReportKind};
use source_span::Span;
use std::fmt::{self, Debug};
use std::ops::Range;
use std::path::{Path, PathBuf};
use trilogy_ir::ir::DefinitionItem;
use trilogy_parser::Spanned;

/// A report of the errors and warnings raised when compiling a Trilogy program.
///
/// Use this report to display to end users what is wrong with their code. The
/// report is not intended for handling errors from Rust, as something wrong with
/// Trilogy source code likely cannot be solved without developer intervention.
///
/// The only way to really consume a Report is to print it out using the provided
/// methods.
pub struct Report<E: std::error::Error> {
    relative_base: PathBuf,
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

impl<E: std::error::Error + 'static> Report<E> {
    /// Print this report to standard error.
    ///
    /// This is the intended way of consuming a Report.
    pub fn eprint(&self) {
        let loader = Loader::new(self.cache.as_ref());
        let cache = FnCache::new(move |loc: &&Location| {
            loader
                .load_source(loc)
                .map(|s| s.unwrap())
                .map_err(|e| Box::new(e) as Box<dyn Debug>)
        });
        let mut cache = LoaderCache {
            relative_base: &self.relative_base,
            inner: cache,
        };

        for warning in &self.warnings {
            warning.eprint(&mut cache, ReportKind::Warning);
        }
        for error in &self.errors {
            error.eprint(&mut cache, ReportKind::Error);
        }
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

struct LoaderCache<'a, F> {
    relative_base: &'a Path,
    inner: FnCache<&'a Location, F>,
}

impl<'a, F> ariadne::Cache<&'a Location> for LoaderCache<'a, F>
where
    F: for<'b> FnMut(&'b &'a Location) -> Result<String, Box<dyn Debug>>,
{
    fn fetch(&mut self, id: &&'a Location) -> Result<&ariadne::Source, Box<dyn fmt::Debug + '_>> {
        self.inner.fetch(id)
    }

    fn display<'b>(&self, id: &'b &'a Location) -> Option<Box<dyn fmt::Display + 'a>> {
        if id.as_ref().scheme() != "file" {
            return Some(Box::new(id.to_owned()));
        }
        match Path::new(id.as_ref().path()).strip_prefix(self.relative_base) {
            Ok(path) => Some(Box::new(path.display().to_string())),
            Err(..) => Some(Box::new(id.to_owned())),
        }
    }
}

impl<E: std::error::Error> Error<E> {
    fn eprint<'a, C: ariadne::Cache<&'a Location>>(&'a self, mut cache: C, kind: ReportKind) {
        let mut colors = ColorGenerator::new();
        let primary = colors.next();
        let secondary = colors.next();

        let report = match &self.0 {
            ErrorKind::External(error) => {
                eprintln!("{}", error);
                return;
            }
            ErrorKind::Ir(location, error) => {
                use trilogy_ir::Error::*;
                match error {
                    UnknownExport { name } => {
                        let span = cache.span(location, name.span());
                        ariadne::Report::build(kind, location, span.1.start)
                            .with_message(format!(
                                "exporting undeclared identifier `{}`",
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
                        ariadne::Report::build(kind, location, span.1.start)
                            .with_message(format!(
                                "reference to undeclared identifier `{}`",
                                name.as_ref().fg(primary),
                            ))
                            .with_label(
                                Label::new(span)
                                    .with_color(primary)
                                    .with_message("referenced here"),
                            )
                    }
                    DuplicateDefinition {
                        original,
                        duplicate,
                    } => {
                        let span = cache.span(location, duplicate.span());
                        let original = cache.span(location, *original);
                        ariadne::Report::build(kind, location, span.1.start)
                                .with_message(format!(
                                    "duplicate declaration of `{}` conflicts with {}",
                                    duplicate.as_ref().fg(primary),
                                    "original declaration".fg(secondary),
                                ))
                                .with_label(
                                    Label::new(span)
                                        .with_color(primary)
                                        .with_message("this declaration...")
                                        .with_order(1)
                                )
                                .with_label(
                                    Label::new(original)
                                        .with_color(secondary)
                                        .with_message("... conflicts with the original declaration here")
                                        .with_order(2)
                                )
                                .with_note("all declarations in the same scope with the same name must be of the same type and arity")
                    }
                    IdentifierInOwnDefinition { name } => {
                        let span = cache.span(location, name.span);
                        let declaration_span = cache.span(location, name.declaration_span);
                        ariadne::Report::build(kind, location, span.1.start)
                            .with_message(format!(
                                "declaration of `{}` references itself in its own initializer",
                                name.id.name().unwrap().fg(primary),
                            ))
                            .with_label(
                                Label::new(declaration_span)
                                    .with_color(primary)
                                    .with_message("variable being declared here"),
                            )
                            .with_label(
                                Label::new(span)
                                    .with_color(primary)
                                    .with_message("is referenced in its own initializer"),
                            )
                            .with_config(Config::default().with_cross_gap(false))
                    }
                    AssignedImmutableBinding { name, assignment } => {
                        let span = cache.span(location, *assignment);
                        let declaration_span = cache.span(location, name.declaration_span);
                        ariadne::Report::build(kind, location, span.1.start)
                            .with_message(format!(
                                "cannot reassign immutable variable `{}`",
                                name.id.name().unwrap().fg(primary)
                            ))
                            .with_label(
                                Label::new(declaration_span)
                                    .with_color(primary)
                                    .with_message("variable declared immutably"),
                            )
                            .with_label(
                                Label::new(span)
                                    .with_color(primary)
                                    .with_message("is being reassigned here"),
                            )
                            .with_help(format!(
                                "consider making this binding mutable: `mut {}`",
                                name.id.name().unwrap(),
                            ))
                    }
                    NoMainProcedure => ariadne::Report::build(kind, location, 0)
                        .with_message("no definition of `proc main!()` was found"),
                    MainNotProcedure { item } => match item {
                        DefinitionItem::Function(func) => {
                            let span = cache.span(location, func.overloads[0].head_span);
                            ariadne::Report::build(kind, location, span.1.start)
                                .with_message("no definition of `proc main!()` was found")
                                .with_label(Label::new(span).with_color(primary).with_message(
                                    "`func main` was found, but main must be a procedure",
                                ))
                        }
                        DefinitionItem::Constant(constant) => {
                            let span = cache.span(location, constant.name.span);
                            ariadne::Report::build(kind, location, span.1.start)
                                .with_message("no definition of `proc main!()` was found")
                                .with_label(Label::new(span).with_color(primary).with_message(
                                    "`const main` was found, but main must be a procedure",
                                ))
                        }
                        DefinitionItem::Rule(rule) => {
                            let span = cache.span(location, rule.overloads[0].head_span);
                            ariadne::Report::build(kind, location, span.1.start)
                                .with_message("no definition of `proc main!()` was found")
                                .with_label(Label::new(span).with_color(primary).with_message(
                                    "`rule main` was found, but main must be a procedure",
                                ))
                        }
                        DefinitionItem::Module(module) => {
                            let span = cache.span(location, module.name.span);
                            ariadne::Report::build(kind, location, span.1.start)
                                .with_message("no definition of `proc main!()` was found")
                                .with_label(Label::new(span).with_color(primary).with_message(
                                    "`module main` was found, but main must be a procedure",
                                ))
                        }
                        DefinitionItem::Test(..) => unreachable!(),
                        DefinitionItem::Procedure(..) => unreachable!(),
                    },
                }
            }
            ErrorKind::Syntax(location, error) => {
                let span = cache.span(location, error.span());
                ariadne::Report::build(kind, location, span.1.start)
                    .with_message(error.message())
                    .with_label(Label::new(span).with_color(primary))
            }
            ErrorKind::Resolver(location, error) => {
                let span = cache.span(location, error.span);
                ariadne::Report::build(kind, location, span.1.start)
                    .with_message(format!(
                        "module resolution failed for module {}: {error}",
                        error.location.as_ref().fg(primary)
                    ))
                    .with_label(
                        Label::new(span)
                            .with_message("module referenced here")
                            .with_color(primary),
                    )
            }
        };
        report.finish().eprint(cache).unwrap();
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

    pub fn report<C: Cache<Error = E> + 'static>(
        self,
        relative_base: PathBuf,
        cache: C,
    ) -> Report<E> {
        Report {
            relative_base,
            cache: Box::new(cache),
            errors: self.errors,
            warnings: self.warnings,
        }
    }

    pub fn checkpoint<C: Cache<Error = E> + 'static>(
        &mut self,
        relative_base: &Path,
        cache: C,
    ) -> Result<C, Report<E>> {
        if self.has_errors() {
            Err(std::mem::take(self).report(relative_base.to_owned(), cache))
        } else {
            Ok(cache)
        }
    }
}
