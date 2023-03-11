use crate::Module;
use std::collections::HashMap;
use trilogy_analyzer::{Analyzer, LexicalError};
use trilogy_lexical_ir as ir;
use trilogy_parser::syntax::Document;
use trilogy_parser::Parse;
use url::Url;

#[derive(Clone, Default, Debug)]
pub struct Binder<T> {
    pub modules: HashMap<Url, Module<T>>,
}

impl Binder<Parse<Document>> {
    pub fn analyze(self) -> Result<Binder<ir::Module>, Vec<LexicalError>> {
        let mut errors = vec![];
        let mut updated = HashMap::new();
        for (url, module) in self.modules {
            let upgraded = module.upgrade(|contents| {
                let ast = contents.into_ast();
                let analyzer = Analyzer::new();
                let analysis = analyzer.analyze(ast);
                errors.extend(analysis.errors);
                analysis.module
            });
            updated.insert(url, upgraded);
        }
        if errors.is_empty() {
            Ok(Binder { modules: updated })
        } else {
            Err(errors)
        }
    }
}
