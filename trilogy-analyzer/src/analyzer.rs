use crate::{Analysis, LexicalError, Scope};
use trilogy_parser::syntax::Document;

#[derive(Default)]
pub struct Analyzer {
    errors: Vec<LexicalError>,
    scope: Scope,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            errors: vec![],
            scope: Scope::default(),
        }
    }

    pub fn analyze(mut self, document: Document) -> Analysis {
        let module = crate::analyze::analyze(&mut self, document);

        Analysis {
            module,
            errors: self.errors,
        }
    }

    pub(crate) fn error(&mut self, error: LexicalError) {
        self.errors.push(error);
    }

    pub(crate) fn scope_mut(&mut self) -> &mut Scope {
        &mut self.scope
    }

    pub(crate) fn scope(&self) -> &Scope {
        &self.scope
    }
}
