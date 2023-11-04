use trilogy_ir::ir;

#[derive(Debug)]
pub enum ErrorKind {
    NoMainProcedure,
    MainHasParameters { proc: ir::ProcedureDefinition },
    MainNotProcedure { item: ir::DefinitionItem },
}
