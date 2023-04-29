use crate::ir::Module;
use std::sync::Arc;

pub trait Resolver {
    fn resolve(&mut self, path: &str) -> Arc<Module>;
}
