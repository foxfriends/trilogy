use super::Error;
use super::error::ErrorKind;
use super::loader::Loader;
use crate::Cache;
use crate::ariadne::{CacheExt, LoaderCache};
use crate::location::Location;
use ariadne::{ColorGenerator, Config, Fmt, FnCache, Label, ReportKind};
use std::collections::HashMap;
use std::fmt::{self, Debug};
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
    libraries: HashMap<Location, String>,
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
        // NOTE: errors in libraries are unexpected, and cannot be reported accurately at this time
        let loader = Loader::new(self.cache.as_ref(), &self.libraries);
        let cache = FnCache::new(move |loc: &Location| {
            loader
                .load_source(loc)
                .map(|s| s.unwrap_or_else(|| "".to_owned()))
                .map_err(|e| Box::new(e) as Box<dyn Debug>)
        });
        let mut cache = LoaderCache::<_, String>::new(&self.relative_base, cache);

        for warning in &self.warnings {
            warning.eprint(&mut cache, ReportKind::Warning);
        }
        for error in &self.errors {
            error.eprint(&mut cache, ReportKind::Error);
        }
    }
}

impl<E: std::error::Error> Error<E> {
    fn eprint<C: ariadne::Cache<Location>>(&self, mut cache: C, kind: ReportKind) {
        let mut colors = ColorGenerator::new();
        let primary = colors.next();
        let secondary = colors.next();
        let tertiary = colors.next();

        let report = match &self.0 {
            ErrorKind::External(error) => {
                eprintln!("{error}");
                return;
            }
            ErrorKind::Ir(location, error) => {
                use trilogy_ir::Error;
                match error {
                    Error::Unimplemented { feature, span } => {
                        let span = cache.span(location, *span);
                        ariadne::Report::build(kind, span.clone())
                            .with_message(format!(
                                "feature `{}` is not implemented",
                                feature.fg(primary)
                            ))
                            .with_label(
                                Label::new(span)
                                    .with_color(primary)
                                    .with_message("used here"),
                            )
                    }
                    Error::UnknownExport { name } => {
                        let span = cache.span(location, name.span());
                        ariadne::Report::build(kind, span.clone())
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
                    Error::UnboundIdentifier { name } => {
                        let span = cache.span(location, name.span());
                        ariadne::Report::build(kind, span.clone())
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
                    Error::DuplicateDefinition {
                        original,
                        duplicate,
                    } => {
                        let span = cache.span(location, duplicate.span());
                        let original = cache.span(location, *original);
                        ariadne::Report::build(kind, span.clone())
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
                    Error::DuplicateExport {
                        original,
                        duplicate,
                    } => {
                        let span = cache.span(location, duplicate.span());
                        let original = cache.span(location, *original);
                        ariadne::Report::build(kind, span.clone())
                            .with_message(format!(
                                "identifier `{}` has already been exported",
                                duplicate.as_ref().fg(primary),
                            ))
                            .with_label(
                                Label::new(span)
                                    .with_color(primary)
                                    .with_message("this export...")
                                    .with_order(1),
                            )
                            .with_label(
                                Label::new(original)
                                    .with_color(secondary)
                                    .with_message("... was already listed here")
                                    .with_order(2),
                            )
                    }
                    Error::IdentifierInOwnDefinition { name } => {
                        let span = cache.span(location, name.span);
                        let declaration_span = cache.span(location, name.declaration_span);
                        ariadne::Report::build(kind, span.clone())
                            .with_message(format!(
                                "declaration of `{}` references itself in its own initializer",
                                name.id.name().fg(primary),
                            ))
                            .with_label(
                                Label::new(declaration_span)
                                    .with_color(primary)
                                    .with_message("variable being declared here")
                                    .with_order(1),
                            )
                            .with_label(
                                Label::new(span)
                                    .with_color(primary)
                                    .with_message("is referenced in its own initializer")
                                    .with_order(2),
                            )
                            .with_config(Config::default().with_cross_gap(false))
                    }
                    Error::AssignedImmutableBinding { name, assignment } => {
                        let span = cache.span(location, *assignment);
                        let declaration_span = cache.span(location, name.declaration_span);
                        ariadne::Report::build(kind, span.clone())
                            .with_message(format!(
                                "cannot reassign immutable variable `{}`",
                                name.id.name().fg(primary)
                            ))
                            .with_label(
                                Label::new(declaration_span)
                                    .with_color(primary)
                                    .with_message("variable declared immutably")
                                    .with_order(1),
                            )
                            .with_label(
                                Label::new(span)
                                    .with_color(primary)
                                    .with_message("is being reassigned here")
                                    .with_order(2),
                            )
                            .with_help(format!(
                                "consider making this binding mutable: `mut {}`",
                                name.id.name(),
                            ))
                    }
                    Error::GluePatternMissingLiteral { lhs, glue, rhs } => {
                        let lhs = cache.span(location, *lhs);
                        let glue = cache.span(location, *glue);
                        let rhs = cache.span(location, *rhs);
                        ariadne::Report::build(kind, glue.clone())
                            .with_message(
                                "at least one side of a glue pattern must be a string literal",
                            )
                            .with_label(
                                Label::new(glue)
                                    .with_message("in this glue pattern")
                                    .with_color(primary)
                                    .with_order(1),
                            )
                            .with_label(
                                Label::new(lhs)
                                    .with_message("neither the left hand side")
                                    .with_color(secondary)
                                    .with_order(2),
                            )
                            .with_label(
                                Label::new(rhs)
                                    .with_message("nor the right hand side is a string literal")
                                    .with_color(tertiary)
                                    .with_order(3),
                            )
                    }
                    Error::NonConstantExpressionInConstant { expression } => {
                        let span = cache.span(location, *expression);
                        ariadne::Report::build(kind, span.clone())
                            .with_message(
                                "only constant expressions are allowed in constant definitions",
                            )
                            .with_label(
                                Label::new(span)
                                    .with_message("in this expression")
                                    .with_color(primary)
                                    .with_order(1),
                            )
                    }
                    Error::NoReturnFromRule { expression } => {
                        let span = cache.span(location, *expression);
                        ariadne::Report::build(kind, span.clone())
                            .with_message(
                                "use of the return keyword is not valid in the body of a rule",
                            )
                            .with_label(
                                Label::new(span)
                                    .with_message("in this expression")
                                    .with_color(primary)
                                    .with_order(1),
                            )
                    }
                    Error::MultiValuedPatternInSet { expression } => {
                        let span = cache.span(location, *expression);
                        ariadne::Report::build(kind, span.clone())
                            .with_message(
                                "the elements of a set pattern may only bind to a single value",
                            )
                            .with_label(
                                Label::new(span)
                                    .with_message(
                                        "this pattern can possibly match more than one value",
                                    )
                                    .with_color(primary)
                                    .with_order(1),
                            )
                    }
                    Error::MultiValuedPatternInRecordKey { expression } => {
                        let span = cache.span(location, *expression);
                        ariadne::Report::build(kind, span.clone())
                            .with_message(
                                "the keys of a record pattern may only bind to a single value",
                            )
                            .with_label(
                                Label::new(span)
                                    .with_message(
                                        "this pattern can possibly match more than one value",
                                    )
                                    .with_color(primary)
                                    .with_order(1),
                            )
                    }
                    Error::BreakOutsideLoopContext { span } => {
                        let span = cache.span(location, *span);
                        ariadne::Report::build(kind, span.clone())
                            .with_message("break may not be used outside of a loop")
                            .with_label(
                                Label::new(span)
                                    .with_message("break used here")
                                    .with_color(primary)
                                    .with_order(1),
                            )
                    }
                    Error::ContinueOutsideLoopContext { span } => {
                        let span = cache.span(location, *span);
                        ariadne::Report::build(kind, span.clone())
                            .with_message("continue may not be used outside of a loop")
                            .with_label(
                                Label::new(span)
                                    .with_message("continue used here")
                                    .with_color(primary)
                                    .with_order(1),
                            )
                    }
                    Error::CancelOutsideHandlerContext { span } => {
                        let span = cache.span(location, *span);
                        ariadne::Report::build(kind, span.clone())
                            .with_message("cancel may not be used outside of a handler")
                            .with_label(
                                Label::new(span)
                                    .with_message("cancel used here")
                                    .with_color(primary)
                                    .with_order(1),
                            )
                    }
                    Error::ResumeOutsideHandlerContext { span } => {
                        let span = cache.span(location, *span);
                        ariadne::Report::build(kind, span.clone())
                            .with_message("resume may not be used outside of a handler")
                            .with_label(
                                Label::new(span)
                                    .with_message("resume used here")
                                    .with_color(primary)
                                    .with_order(1),
                            )
                    }
                    Error::BecomeOutsideHandlerContext { span } => {
                        let span = cache.span(location, *span);
                        ariadne::Report::build(kind, span.clone())
                            .with_message("become may not be used outside of a handler")
                            .with_label(
                                Label::new(span)
                                    .with_message("become used here")
                                    .with_color(primary)
                                    .with_order(1),
                            )
                    }
                }
            }
            ErrorKind::Analysis(location, error) => {
                use super::analyzer::ErrorKind;
                match error {
                    ErrorKind::NoMainProcedure => {
                        ariadne::Report::build(kind, (location.clone(), 0..0))
                            .with_message("no definition of `proc main!()` was found")
                    }
                    ErrorKind::MainHasParameters { proc } => {
                        let span = cache.span(location, proc.overloads[0].span);
                        ariadne::Report::build(kind, span.clone())
                            .with_message("definition of `proc main!()` must not accept parameters")
                            .with_label(Label::new(span).with_color(primary).with_message(format!(
                                "procedure accepts {} parameters",
                                proc.overloads[0].parameters.len()
                            )))
                    }
                    ErrorKind::MainNotProcedure { item } => match item {
                        DefinitionItem::Function(func) => {
                            let span = cache.span(location, func.overloads[0].head_span);
                            ariadne::Report::build(kind, span.clone())
                                .with_message("no definition of `proc main!()` was found")
                                .with_label(Label::new(span).with_color(primary).with_message(
                                    "`func main` was found, but main must be a procedure",
                                ))
                        }
                        DefinitionItem::Constant(constant) => {
                            let span = cache.span(location, constant.name.span);
                            ariadne::Report::build(kind, span.clone())
                                .with_message("no definition of `proc main!()` was found")
                                .with_label(Label::new(span).with_color(primary).with_message(
                                    "`const main` was found, but main must be a procedure",
                                ))
                        }
                        DefinitionItem::Rule(rule) => {
                            let span = cache.span(location, rule.overloads[0].head_span);
                            ariadne::Report::build(kind, span.clone())
                                .with_message("no definition of `proc main!()` was found")
                                .with_label(Label::new(span).with_color(primary).with_message(
                                    "`rule main` was found, but main must be a procedure",
                                ))
                        }
                        DefinitionItem::Module(module) => {
                            let span = cache.span(location, module.name.span);
                            ariadne::Report::build(kind, span.clone())
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
                use trilogy_parser::syntax::ErrorKind;
                let span = cache.span(location, error.span());
                match error.kind() {
                    ErrorKind::Unknown(message) => ariadne::Report::build(kind, span.clone())
                        .with_message(message)
                        .with_label(Label::new(span).with_color(primary)),
                    ErrorKind::RuleRightArrow => ariadne::Report::build(kind, span.clone())
                        .with_message(format!("a `{}` may not be used in a rule definition", "->".fg(primary)))
                        .with_label(Label::new(span).with_color(primary).with_message("try replacing it with a `<-`")),
                    ErrorKind::KwNotInExpression => ariadne::Report::build(kind, span.clone())
                        .with_message(format!("the `{}` keyword may not be used in an expression, did you mean to use the `!` operator?", "not".fg(primary)))
                        .with_label(Label::new(span).with_color(primary).with_message("try replacing this `not` with `!`")),
                    ErrorKind::MatchStatementExpressionCase => ariadne::Report::build(kind, span.clone())
                        .with_message("cases in a match statement must be handled with blocks")
                        .with_label(Label::new(span).with_color(primary).with_message("try replacing this handler with a block")),
                    ErrorKind::TripleDot { dot } => {
                        let dot = cache.span(location, *dot);
                        ariadne::Report::build(kind, span.clone())
                            .with_message(format!("unexpected extra `{}` in spread (`{}`) expression", ".".fg(primary), "..".fg(secondary)))
                            .with_label(Label::new(span).with_color(secondary).with_message("in this spread expression"))
                            .with_label(Label::new(dot).with_color(primary).with_message("try removing this `.`"))
                            .with_help("the spread operator uses only two (`..`)")
                    }
                    ErrorKind::IfExpressionRestriction => ariadne::Report::build(kind, span)
                        .with_message("an `if` expression must have an `else` clause"),
                    ErrorKind::TaggedTemplateMissingIdentifier => ariadne::Report::build(kind, span.clone())
                        .with_message("a tagged template requires a tag identifier")
                        .with_label(Label::new(span).with_color(primary).with_message("try inserting an identifier here")),
                    ErrorKind::TaggedTemplateNotIdentifier => ariadne::Report::build(kind, span.clone())
                        .with_message("the `$` operator prefixing a tagged template requires an identifier")
                        .with_label(Label::new(span).with_color(primary).with_message("this must be an identifier")),
                    ErrorKind::DoMissingParameterList => ariadne::Report::build(kind, span.clone())
                        .with_message("a `do` closure requires a parameter list, even if empty")
                        .with_label(Label::new(span).with_color(primary).with_message("try adding `()` after this `do`"))
                }
            }
            ErrorKind::Resolver(location, error) => {
                let span = cache.span(location, error.span);
                ariadne::Report::build(kind, span.clone())
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
    libraries: HashMap<Location, String>,
}

impl<E: std::error::Error> Default for ReportBuilder<E> {
    fn default() -> Self {
        Self {
            errors: vec![],
            warnings: vec![],
            libraries: HashMap::default(),
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

    pub fn add_libraries(&mut self, libraries: HashMap<Location, String>) {
        self.libraries.extend(libraries);
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
            libraries: self.libraries.clone(),
        }
    }

    pub fn checkpoint<C: Cache<Error = E> + 'static>(
        &mut self,
        relative_base: &Path,
        cache: C,
    ) -> Result<C, Box<Report<E>>> {
        if self.has_errors() {
            Err(Box::new(
                std::mem::take(self).report(relative_base.to_owned(), cache),
            ))
        } else {
            Ok(cache)
        }
    }
}
