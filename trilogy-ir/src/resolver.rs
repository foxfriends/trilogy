use crate::ir::ModuleCell;
use std::sync::Arc;

pub trait Resolver {
    fn location(&self) -> String;
    fn resolve(&mut self, path: &str) -> Arc<ModuleCell>;
}
