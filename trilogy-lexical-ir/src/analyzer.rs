use crate::ir::Module;
use crate::{Analysis, LexicalError};
use trilogy_parser::syntax::Document;

#[derive(Default)]
pub struct Analyzer {
    errors: Vec<LexicalError>,
}

impl Analyzer {
    pub fn new() -> Self {
        Self { errors: vec![] }
    }

    pub fn analyze(mut self, document: Document) -> Analysis {
        let module = Module::analyze(&mut self, document);

        Analysis {
            module,
            errors: self.errors,
        }
    }

    pub(crate) fn error(&mut self, error: LexicalError) {
        self.errors.push(error);
    }
}
