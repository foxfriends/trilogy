use crate::codegen::{Codegen, Head};
use inkwell::module::Linkage;
use trilogy_ir::ir::{self, DefinitionItem};

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn compile_module(&self, file: &str, module: &ir::Module) -> Codegen<'ctx> {
        let mut subcontext = self.for_file(file);
        Self::compile_module_contents(&mut subcontext, module);
        subcontext
    }

    pub(crate) fn compile_submodule(subcontext: &mut Codegen<'ctx>, module: &ir::Module) {
        Self::compile_module_contents(subcontext, module);
    }

    fn compile_module_contents(subcontext: &mut Codegen<'ctx>, module: &ir::Module) {
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
                        .insert(module.name.id.clone(), Head::ExternalModule(location));
                }
                DefinitionItem::Module(def) => {
                    let module = def.module.as_module().unwrap();
                    if module.parameters.is_empty() {
                        subcontext.declare_constant(
                            &def.name.to_string(),
                            definition.is_exported,
                            definition.span,
                        );
                        subcontext.globals.insert(def.name.id.clone(), Head::Module);
                    } else {
                        subcontext.declare_function(
                            &def.name.to_string(),
                            if definition.is_exported {
                                Linkage::External
                            } else {
                                Linkage::Private
                            },
                            definition.span,
                        );
                        subcontext
                            .globals
                            .insert(def.name.id.clone(), Head::Function);
                    }
                }
                DefinitionItem::Constant(constant) => {
                    subcontext.declare_constant(
                        &constant.name.to_string(),
                        definition.is_exported,
                        definition.span,
                    );
                    subcontext
                        .globals
                        .insert(constant.name.id.clone(), Head::Constant);
                }
                DefinitionItem::Procedure(procedure) if procedure.overloads.is_empty() => {
                    subcontext.declare_extern_procedure(
                        &procedure.name.to_string(),
                        procedure.arity,
                        if definition.is_exported {
                            Linkage::External
                        } else {
                            Linkage::Private
                        },
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
                        if definition.is_exported {
                            Linkage::External
                        } else {
                            Linkage::Private
                        },
                        procedure.span(),
                    );
                    subcontext
                        .globals
                        .insert(procedure.name.id.clone(), Head::Procedure);
                }
                DefinitionItem::Function(function) => {
                    subcontext.declare_function(
                        &function.name.to_string(),
                        if definition.is_exported {
                            Linkage::External
                        } else {
                            Linkage::Private
                        },
                        function.span(),
                    );
                    subcontext
                        .globals
                        .insert(function.name.id.clone(), Head::Function);
                }
                DefinitionItem::Rule(..) => todo!("implement rule"),
                DefinitionItem::Test(..) => {}
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
                DefinitionItem::Module(def) => {
                    let module = def.module.as_module().unwrap();
                    subcontext.begin_submodule(def.name.to_string());
                    Self::compile_submodule(subcontext, module);
                    subcontext.end_submodule();
                }
                DefinitionItem::Constant(constant) => {
                    subcontext.compile_constant(constant);
                }
                DefinitionItem::Rule(..) => todo!("implement rule"),
                DefinitionItem::Test(..) => {}
            }
        }
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
