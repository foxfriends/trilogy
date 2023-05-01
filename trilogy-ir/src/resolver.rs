use crate::ir::ModuleCell;
use std::sync::Arc;

pub trait Resolver {
    fn resolve(&mut self, path: &str) -> Arc<ModuleCell>;
}
