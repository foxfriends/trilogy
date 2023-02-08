use crate::syntax::{Document, SyntaxError};

#[derive(Clone, Debug)]
pub struct Parse {
    pub(crate) ast: Document,
    pub(crate) warnings: Vec<SyntaxError>,
    pub(crate) errors: Vec<SyntaxError>,
}

impl Parse {
    pub fn ast(&self) -> &Document {
        &self.ast
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
