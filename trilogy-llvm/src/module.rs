use std::collections::HashMap;

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
            modules: self.modules,
            external_modules: HashMap::new(),
            location: name.to_owned(),
        }
    }

    pub(crate) fn compile_module(&self, file: &str, module: &ir::Module) -> Codegen<'ctx> {
        let mut subcontext = self.sub(file);

        // Pre-declare everything this module will reference so that all references during codegen will
        // be valid.
        for definition in module.definitions() {
            match &definition.item {
                DefinitionItem::Module(module) if module.module.as_external().is_some() => {
                    let location = module.module.as_external().unwrap().to_owned();
                    if let Some(submodule) = subcontext.modules.get(&location).unwrap() {
                        subcontext.import_module(&location, submodule);
                    } else {
                        subcontext.import_libc();
                    }
                    subcontext
                        .external_modules
                        .insert(module.name.id.clone(), location);
                }
                _ => {}
            }
        }

        // Now comes actual codegen
        for definition in module.definitions() {
            match &definition.item {
                DefinitionItem::Procedure(procedure) => {
                    subcontext.compile_procedure(procedure, definition.is_exported);
                }
                DefinitionItem::Module(module) if module.module.as_external().is_some() => {}
                DefinitionItem::Constant(constant) => {
                    subcontext.compile_constant(constant, definition.is_exported);
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
                    self.import_procedure(location, procedure);
                }
                DefinitionItem::Constant(constant) => {
                    self.import_constant(location, constant);
                }
                _ => todo!(),
            }
        }
    }
}
