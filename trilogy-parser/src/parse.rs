use crate::syntax::{Amble, SyntaxError};

#[derive(Clone, Debug)]
pub struct Parse<T> {
    pub(crate) ast: Amble<T>,
    pub(crate) warnings: Vec<SyntaxError>,
    pub(crate) errors: Vec<SyntaxError>,
}

impl<T> Parse<T> {
    pub fn ast(&self) -> &T {
        &self.ast.content
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn warnings(&self) -> &[SyntaxError] {
        &self.warnings
    }

    pub fn errors(&self) -> &[SyntaxError] {
        &self.errors
    }
}
