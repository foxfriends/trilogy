use colored::{Color, Colorize};
use std::fmt::{self, Debug, Display};
use trilogy_vm::Location;

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

impl std::error::Error for RuntimeError {}

#[derive(Default)]
struct StackFrameNote<'a> {
    definition: Option<(String, &'a Location)>,
    expr: Option<(String, &'a Location)>,
    note: Option<String>,
    source: Option<String>,
}

impl Display for StackFrameNote<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut space = "";
        if let Some((def, loc)) = &self.definition {
            write!(
                f,
                "in {def} ({} {})",
                loc.file.color(Color::Green),
                loc.span.start().to_string().color(Color::Cyan)
            )?;
            space = "\n       ";
        }
        if let Some((note, loc)) = &self.expr {
            write!(
                f,
                "{space}{note} ({})",
                loc.span.start().to_string().color(Color::Cyan)
            )?;
            space = "\n       ";
        }
        if let Some(note) = &self.note {
            write!(f, "{space}{note}")?
        }
        Ok(())
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.error)?;

        writeln!(f, "Stack trace:")?;
        for (i, frame) in self.error.stack_trace.frames.iter().enumerate() {
            let mut note = StackFrameNote::default();

            for (label, location) in &frame.source_annotations {
                if label.starts_with("proc")
                    || label.starts_with("func")
                    || label.starts_with("rule")
                {
                    let (kw, name) = label.split_once(' ').unwrap();
                    let label = format!("{} {}", kw.color(Color::Magenta), name.color(Color::Blue));
                    if let Some((_, orig)) = note.definition {
                        if location < orig {
                            note.definition = Some((label, location));
                        }
                    } else {
                        note.definition = Some((label, location));
                    }
                } else if label == "<intermediate>" {
                    note.expr = Some(("computing intermediate expression".to_owned(), location));
                } else if label == "<entrypoint>" {
                    note.expr = Some(("at program entrypoint".to_owned(), location));
                } else if note.expr.is_none() {
                    note.expr = Some((label.to_owned(), location));
                }
            }
            for label in &frame.notes {
                note.note = Some(label.to_owned());
            }
            writeln!(f, "{i:>5}: {note}")?;
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
