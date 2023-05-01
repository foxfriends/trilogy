use std::sync::Arc;
use trilogy_ir::ir;

#[derive(Debug)]
pub struct Program {
    #[allow(dead_code)]
    module: Arc<ir::ModuleCell>,
}

impl Program {
    pub(crate) fn new(module: Arc<ir::ModuleCell>) -> Self {
        Self { module }
    }
}
