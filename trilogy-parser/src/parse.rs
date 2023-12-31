use crate::syntax::{Amble, SyntaxError};

/// Encapsulates the result of attempting to parse a Trilogy file.
///
/// This is similar to but more than just a [`Result`][], as a Trilogy file may
/// contain multiple errors or warnings.
///
/// While the contents are accessible whether the parse was successful or not,
/// if the `Parse` contains errors, then the exact structure of the contents
/// is not defined to be well formed.
#[derive(Clone, Debug)]
pub struct Parse<T> {
    pub(crate) ast: Amble<T>,
    pub(crate) warnings: Vec<SyntaxError>,
    pub(crate) errors: Vec<SyntaxError>,
}

impl<T> Parse<T> {
    /// A reference to the contents of this parse.
    ///
    /// If this `Parse` contains errors, the exact structure of these contents
    /// are not defined to be well formed.
    pub fn ast(&self) -> &T {
        &self.ast.content
    }

    /// Consume the parse, turning it into its contents.
    ///
    /// If this `Parse` contains errors, the exact structure of these contents
    /// are not defined to be well formed.
    pub fn into_ast(self) -> T {
        self.ast.content
    }

    /// Whether this `Parse` contains errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Whether this `Parse` contains warnings.
    ///
    /// Warnings do not affect the validity of the parsed contents, but may
    /// be presented to the writer of the source code to inform them of potential
    /// mistakes.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// The list of warnings generated during this parse.
    pub fn warnings(&self) -> &[SyntaxError] {
        &self.warnings
    }

    /// The list of errors generated during this parse.
    pub fn errors(&self) -> &[SyntaxError] {
        &self.errors
    }
}
