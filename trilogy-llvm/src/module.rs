use crate::{
    codegen::{Codegen, Head},
    debug_info::DebugInfo,
};
use inkwell::module::Linkage;
use std::{collections::HashMap, path::PathBuf};
use trilogy_ir::ir::{self, DefinitionItem};
use url::Url;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn sub(&self, name: &str) -> Codegen<'ctx> {
        let module = self.context.create_module(name);
        let url = Url::parse(name).unwrap();
        let (filename, directory) = match url.scheme() {
            "file" => {
                let path: PathBuf = url.path().parse().unwrap();
                (
                    path.file_name().unwrap().to_string_lossy().into_owned(),
                    path.parent().unwrap().display().to_string(),
                )
            }
            "http" | "https" => {
                let path: PathBuf = url.path().parse().unwrap();
                (
                    path.file_name().unwrap().to_string_lossy().into_owned(),
                    path.parent().unwrap().display().to_string(),
                )
            }
            "trilogy" => (url.path().to_owned(), "/".to_owned()),
            _ => (name.to_owned(), "/".to_owned()),
        };
        let di = DebugInfo::new(&module, &filename, &directory);
        Codegen {
            atoms: self.atoms.clone(),
            context: self.context,
            builder: self.context.create_builder(),
            di,
            execution_engine: self.execution_engine.clone(),
            module,
            modules: self.modules,
            globals: HashMap::new(),
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
                    let submodule = subcontext.modules.get(&location).unwrap();
                    subcontext.import_module(&location, submodule);
                    subcontext
                        .globals
                        .insert(module.name.id.clone(), Head::Module(location));
                }
                DefinitionItem::Constant(constant) => {
                    subcontext.declare_constant(constant, definition.is_exported, definition.span);
                    subcontext
                        .globals
                        .insert(constant.name.id.clone(), Head::Constant);
                }
                DefinitionItem::Procedure(procedure) => {
                    subcontext.declare_procedure(
                        &procedure.name.to_string(),
                        procedure.arity,
                        if definition.is_exported {
                            Linkage::External
                        } else {
                            Linkage::Private
                        },
                        procedure.overloads.is_empty(),
                        procedure.span(),
                    );
                    subcontext
                        .globals
                        .insert(procedure.name.id.clone(), Head::Procedure(procedure.arity));
                }
                _ => {}
            }
        }

        // Now comes actual codegen
        for definition in module.definitions() {
            match &definition.item {
                DefinitionItem::Procedure(procedure) => {
                    subcontext.compile_procedure(procedure);
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
