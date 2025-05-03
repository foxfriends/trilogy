use crate::codegen::{Codegen, Head};
use inkwell::module::Linkage;
use trilogy_ir::ir::{self, DefinitionItem};

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn compile_module(&self, file: &str, module: &ir::Module) -> Codegen<'ctx> {
        let mut subcontext = self.for_submodule(file);

        // Pre-declare everything this module will reference so that all references during codegen will
        // be valid.
        for definition in module.definitions() {
            match &definition.item {
                DefinitionItem::Module(module) if module.module.as_external().is_some() => {
                    let location = module.module.as_external().unwrap().to_owned();
                    let submodule = subcontext.modules.get(&location).unwrap();
                    subcontext.import_module(&location, submodule);
                    subcontext
                        .globals
                        .insert(module.name.id.clone(), Head::Module(location));
                }
                DefinitionItem::Constant(constant) => {
                    subcontext.declare_constant(constant, definition.span);
                    subcontext
                        .globals
                        .insert(constant.name.id.clone(), Head::Constant);
                }
                DefinitionItem::Procedure(procedure) if procedure.overloads.is_empty() => {
                    subcontext.declare_extern_procedure(
                        &procedure.name.to_string(),
                        procedure.arity,
                        Linkage::External,
                        procedure.span(),
                    );
                    subcontext
                        .globals
                        .insert(procedure.name.id.clone(), Head::Procedure);
                }
                DefinitionItem::Procedure(procedure) => {
                    subcontext.declare_procedure(
                        &procedure.name.to_string(),
                        procedure.arity,
                        Linkage::External,
                        procedure.span(),
                    );
                    subcontext
                        .globals
                        .insert(procedure.name.id.clone(), Head::Procedure);
                }
                DefinitionItem::Function(function) => {
                    subcontext.declare_function(
                        &function.name.to_string(),
                        Linkage::External,
                        function.span(),
                    );
                    subcontext
                        .globals
                        .insert(function.name.id.clone(), Head::Function);
                }

                _ => {}
            }
        }

        // Now comes actual codegen
        for definition in module.definitions() {
            match &definition.item {
                DefinitionItem::Procedure(procedure) if procedure.overloads.is_empty() => {}
                DefinitionItem::Procedure(procedure) => {
                    subcontext.compile_procedure(procedure);
                }
                DefinitionItem::Function(function) => {
                    subcontext.compile_function(function);
                }
                DefinitionItem::Module(module) if module.module.as_external().is_some() => {}
                DefinitionItem::Constant(constant) => {
                    subcontext.compile_constant(constant);
                }
                _ => todo!(),
            }
        }

        subcontext
    }

    pub(crate) fn import_module(&self, location: &str, module: &ir::Module) {
        for definition in module.definitions() {
            if !definition.is_exported {
                continue;
            }
            match &definition.item {
                DefinitionItem::Module(module) if module.module.as_external().is_some() => {}
                DefinitionItem::Module(_module) => {
                    todo!()
                }
                DefinitionItem::Procedure(procedure) => {
                    self.import_procedure(location, &procedure.name.to_string());
                }
                DefinitionItem::Function(function) => {
                    self.import_procedure(location, &function.name.to_string());
                }
                DefinitionItem::Constant(constant) => {
                    self.import_constant(location, constant);
                }
                _ => todo!(),
            }
        }
    }
}
