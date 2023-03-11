use trilogy_lexical_ir::Module;

use crate::LexicalError;

#[derive(Clone, Debug)]
pub struct Analysis {
    pub(crate) module: Module,
    pub(crate) errors: Vec<LexicalError>,
}

impl Analysis {
    pub fn ir(&self) -> &Module {
        &self.module
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn errors(&self) -> &[LexicalError] {
        &self.errors
    }
}
