use crate::Module;
use std::collections::HashMap;
use trilogy_parser::syntax::Document;
use trilogy_parser::Parse;
use url::Url;

#[derive(Clone, Default, Debug)]
pub struct Binder<T> {
    pub modules: HashMap<Url, Module<T>>,
}

impl Binder<Parse<Document>> {
    pub fn analyze(self) -> Result<Binder<()>, Vec<()>> {
        let mut updated = HashMap::new();
        for (url, module) in self.modules {
            let upgraded = module.upgrade(|contents| {
                let _ast = contents.into_ast();
                todo!()
            });
            updated.insert(url, upgraded);
        }
        Ok(Binder { modules: updated })
    }
}
