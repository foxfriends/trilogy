use crate::codegen::Codegen;
use trilogy_ir::ir::{self, DefinitionItem};

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn sub(&self, name: &str) -> Codegen<'ctx> {
        let module = self.context.create_module(name);
        Codegen {
            context: self.context,
            builder: self.context.create_builder(),
            execution_engine: self.execution_engine.clone(),
            module,
        }
    }

    pub(crate) fn compile_module(&self, file: &str, module: &ir::Module) -> Codegen<'ctx> {
        let subcontext = self.sub(file);
        for definition in module.definitions() {
            match &definition.item {
                DefinitionItem::Procedure(procedure) => {
                    subcontext.compile_procedure(file, procedure, definition.is_exported);
                }
                DefinitionItem::Module(module) if module.module.as_external().is_some() => {}
                DefinitionItem::Constant(constant) => {
                    subcontext.compile_constant(file, constant, definition.is_exported);
                }
                _ => todo!(),
            }
        }
        subcontext
    }
}
