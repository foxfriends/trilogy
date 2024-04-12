use crate::ariadne::CacheExt;
use ariadne::{ColorGenerator, FnCache, Label, ReportKind};
use colored::Colorize;
use std::fmt::{self, Debug, Display};
use trilogy_vm::Location;
use url::Url;

/// A black box of failure that occurred during the execution of a Trilogy program.
///
/// Such an error might be a runtime error thrown by the program being executed, or
/// an error that occurred within the virtual machine itself, likely from attempting
/// to run invalid bytecode.
///
/// Language runtime errors may be unwrapped and inspected, but internal errors are
/// inaccessible.
pub struct RuntimeError {
    pub(super) error: trilogy_vm::Error,
}

impl From<trilogy_vm::Error> for RuntimeError {
    fn from(error: trilogy_vm::Error) -> Self {
        Self { error }
    }
}

impl std::error::Error for RuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

impl RuntimeError {
    /// An iterator over the stack frames that make up the stack of this runtime error.
    ///
    /// It is recommended to print these frames to the user, while also adding any
    /// additional contex that is known by your environment, if possible.
    fn frames(&self) -> impl Iterator<Item = StackFrameNote<'_>> {
        self.error
            .stack_trace
            .frames
            .iter()
            .enumerate()
            .map(|(i, frame)| {
                let mut note = StackFrameNote {
                    index: i,
                    ..Default::default()
                };

                for (label, location) in &frame.source_annotations {
                    if label.starts_with("proc")
                        || label.starts_with("func")
                        || label.starts_with("rule")
                    {
                        let (kw, name) = label.split_once(' ').unwrap();
                        let label = format!("{} {}", kw.magenta(), name.blue());
                        if let Some((_, orig)) = note.definition {
                            if location < orig {
                                note.definition = Some((label, location));
                            }
                        } else {
                            note.definition = Some((label, location));
                        }
                    } else if label == "<intermediate>" {
                        if let Some((_, orig)) = note.expr {
                            if location < orig {
                                note.expr = Some((
                                    "computing intermediate expression".to_owned(),
                                    location,
                                ));
                            }
                        } else {
                            note.expr =
                                Some(("computing intermediate expression".to_owned(), location));
                        }
                    } else if note.expr.is_none() {
                        note.expr = Some((label.to_owned(), location));
                    }
                }
                for label in &frame.notes {
                    note.note = Some(label.to_owned());
                }
                note
            })
    }

    /// Prints this error to stderr.
    ///
    /// Printing via method includes more information than printing directly using
    /// [`Display`][] would, at the expense of it doing much more work.
    pub fn eprint(&self) {
        eprintln!("Stack trace:");

        let mut colors = ColorGenerator::new();
        let primary = colors.next();
        let mut cache = FnCache::<String, _, String>::new(
            move |path: &String| -> Result<String, Box<dyn Debug>> {
                std::fs::read_to_string(path).map_err(|e| Box::new(e) as Box<dyn Debug>)
            },
        );

        let mut report = None;
        for frame in self.frames() {
            eprintln!("{frame}");

            if report.is_none() {
                if let Some((_, loc)) = &frame.expr {
                    if let Some(local) = loc
                        .file
                        .parse::<Url>()
                        .ok()
                        .and_then(|url| url.to_file_path().ok())
                        .map(|path| path.display().to_string())
                    // Ariadne hates us
                    {
                        let span = cache.span(&local, loc.span);
                        report = Some(
                            ariadne::Report::build(ReportKind::Error, &loc.file, span.1.start)
                                .with_message(format!("{}", self.error))
                                .with_label(
                                    Label::new(span)
                                        .with_color(primary)
                                        .with_message("in this expression"),
                                ),
                        );
                    }
                }
            }
        }

        if let Some(report) = report {
            println!();
            report.finish().eprint(&mut cache).unwrap();
        } else {
            println!("{}", self.error);
        }
    }
}

#[derive(Default)]
pub struct StackFrameNote<'a> {
    index: usize,
    pub definition: Option<(String, &'a Location)>,
    pub expr: Option<(String, &'a Location)>,
    pub note: Option<String>,
}

impl Display for StackFrameNote<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut space = "";
        write!(f, "{:>5}: ", self.index)?;
        if let Some((def, loc)) = &self.definition {
            write!(
                f,
                "in {def} ({} {})",
                loc.file.green(),
                loc.span.start().to_string().cyan()
            )?;
            space = "\n       ";
        }
        if let Some((note, loc)) = &self.expr {
            write!(f, "{space}{note} ({})", loc.span.start().to_string().cyan())?;
            space = "\n       ";
        }
        if let Some(note) = &self.note {
            if note.starts_with("runtime panicked") {
                write!(f, "{space}{}", note.red())?
            } else {
                write!(f, "{space}{note}")?
            }
        }
        Ok(())
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.error)?;

        writeln!(f, "Stack trace:")?;
        for note in self.frames() {
            writeln!(f, "{note}")?;
        }

        Ok(())
    }
}

impl Debug for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.error)?;
        writeln!(f, "Final IP: {}", self.error.ip)?;
        write!(f, "Stack Dump:\n{}", self.error.dump())?;
        Ok(())
    }
}
