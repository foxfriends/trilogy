use trilogy_ir::ir;

#[derive(Debug)]
pub enum ErrorKind {
    NoMainProcedure,
    MainHasParameters { proc: ir::ProcedureDefinition },
    MainNotProcedure { item: ir::DefinitionItem },

    ConstantCycle { def: ir::ConstantDefinition },
    ModuleCycle { def: ir::ModuleDefinition },
}
