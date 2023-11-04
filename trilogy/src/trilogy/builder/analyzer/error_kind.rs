use trilogy_ir::ir;

#[derive(Debug)]
pub enum ErrorKind {
    NoMainProcedure,
    MainNotProcedure { item: ir::DefinitionItem },
}
