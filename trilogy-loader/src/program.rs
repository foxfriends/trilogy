use trilogy_ir::ir;

#[derive(Debug)]
pub struct Program {
    #[allow(dead_code)]
    modules: Vec<ir::Module>,
}

impl Program {
    pub(crate) fn new() -> Self {
        Self { modules: vec![] }
    }
}
