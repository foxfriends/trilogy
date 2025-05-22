use std::collections::BTreeMap;

use crate::codegen::{Codegen, Global, Head};
use inkwell::AddressSpace;
use inkwell::debug_info::AsDIScope;
use inkwell::llvm_sys::LLVMCallConv;
use inkwell::llvm_sys::debuginfo::LLVMDIFlagPublic;
use inkwell::module::Linkage;
use inkwell::values::FunctionValue;
use trilogy_ir::Id;
use trilogy_ir::ir::{self, DefinitionItem};

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn compile_module(&self, file: &str, module: &ir::Module) -> Codegen<'ctx> {
        let mut subcontext = self.for_file(file);
        Self::compile_module_contents(&mut subcontext, module, true);
        subcontext
    }

    pub(crate) fn compile_submodule(
        subcontext: &mut Codegen<'ctx>,
        module: &ir::Module,
        is_public: bool,
    ) {
        Self::compile_module_contents(subcontext, module, is_public);
    }

    fn add_global(&mut self, id: Id, head: Head) {
        self.globals.insert(
            id,
            Global {
                path: self.path.clone(),
                head,
            },
        );
    }

    fn begin_submodule(&mut self, name: String) {
        self.path.push(name);
    }

    fn end_submodule(&mut self) {
        self.path.pop().unwrap();
        self.globals.retain(|_, v| self.path.starts_with(&v.path));
    }

    fn compile_module_contents(
        subcontext: &mut Codegen<'ctx>,
        module: &ir::Module,
        is_public: bool,
    ) {
        // Pre-declare everything this module will reference so that all references during codegen will
        // be valid.
        let mut members = BTreeMap::new();
        for definition in module.definitions() {
            let function = match &definition.item {
                DefinitionItem::Module(module) if module.module.as_external().is_some() => {
                    let location = module.module.as_external().unwrap().to_owned();
                    let submodule = subcontext.modules.get(&location).unwrap();
                    subcontext.add_global(
                        module.name.id.clone(),
                        Head::ExternalModule(location.clone()),
                    );
                    subcontext.import_module(definition.name().unwrap(), &location, submodule)
                }
                DefinitionItem::Module(def) => {
                    let module = def.module.as_module().unwrap();
                    if module.parameters.is_empty() {
                        subcontext.add_global(def.name.id.clone(), Head::Module);
                        subcontext.declare_constant(
                            &def.name.to_string(),
                            definition.is_exported,
                            definition.span,
                        )
                    } else {
                        subcontext.add_global(def.name.id.clone(), Head::Function);
                        subcontext.declare_function(
                            &def.name.to_string(),
                            if definition.is_exported {
                                Linkage::External
                            } else {
                                Linkage::Private
                            },
                            definition.span,
                        )
                    }
                }
                DefinitionItem::Constant(constant) => {
                    subcontext.add_global(constant.name.id.clone(), Head::Constant);
                    subcontext.declare_constant(
                        &constant.name.to_string(),
                        definition.is_exported,
                        definition.span,
                    )
                }
                DefinitionItem::Procedure(procedure) if procedure.overloads.is_empty() => {
                    subcontext.add_global(procedure.name.id.clone(), Head::Procedure);
                    subcontext.declare_extern_procedure(
                        &procedure.name.to_string(),
                        procedure.arity,
                        if definition.is_exported {
                            Linkage::External
                        } else {
                            Linkage::Private
                        },
                        procedure.span(),
                    )
                }
                DefinitionItem::Procedure(procedure) => {
                    subcontext.add_global(procedure.name.id.clone(), Head::Procedure);
                    subcontext.declare_procedure(
                        &procedure.name.to_string(),
                        procedure.arity,
                        if definition.is_exported {
                            Linkage::External
                        } else {
                            Linkage::Private
                        },
                        procedure.span(),
                    )
                }
                DefinitionItem::Function(function) => {
                    subcontext.add_global(function.name.id.clone(), Head::Function);
                    subcontext.declare_function(
                        &function.name.to_string(),
                        if definition.is_exported {
                            Linkage::External
                        } else {
                            Linkage::Private
                        },
                        function.span(),
                    )
                }
                DefinitionItem::Rule(..) => todo!("implement rule"),
                DefinitionItem::Test(..) => continue,
            };
            let member_id = subcontext.atom_value_raw(definition.name().unwrap().to_string());
            members.insert(member_id, function);
        }

        // With all those pre-declared, build the constructor for this module.
        if module.parameters.is_empty() {
            // A module with no parameters comes across as a constant:
            subcontext.compile_constant_constructor(module, members, is_public);
        } else {
            todo!("implement functor modules");
        }

        // Then comes actual codegen
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
                    Self::compile_submodule(subcontext, module, definition.is_exported);
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

    fn compile_constant_constructor(
        &self,
        module: &ir::Module,
        members: BTreeMap<u64, FunctionValue<'ctx>>,
        is_public: bool,
    ) {
        // add_constant
        let linkage = if is_public {
            Linkage::External
        } else {
            Linkage::Private
        };
        let name = self.module_path();
        let accessor = self
            .module
            .add_function(&name, self.accessor_type(), Some(linkage));
        accessor.set_call_conventions(LLVMCallConv::LLVMFastCallConv as u32);

        // declare_constant
        let subprogram = self.di.builder.create_function(
            self.di.unit.as_debug_info_scope(),
            &name,
            None,
            self.di.unit.get_file(),
            module.span.start().line as u32 + 1,
            self.di.procedure_di_type(0),
            linkage != Linkage::External,
            true,
            module.span.start().line as u32 + 1,
            LLVMDIFlagPublic,
            false,
        );
        accessor.set_subprogram(subprogram);

        // The member IDs array is just a constant array
        let member_ids_global = self.module.add_global(
            self.context.i64_type().array_type(members.len() as u32),
            None,
            "",
        );
        member_ids_global.set_initializer(
            &self.context.i64_type().const_array(
                &members
                    .keys()
                    .map(|k| self.context.i64_type().const_int(*k, false))
                    .collect::<Vec<_>>(),
            ),
        );

        // So is the member accessors array
        let members_global = self.module.add_global(
            self.context
                .ptr_type(AddressSpace::default())
                .array_type(members.len() as u32),
            None,
            "",
        );
        members_global.set_initializer(
            &self.context.ptr_type(AddressSpace::default()).const_array(
                &members
                    .values()
                    .map(|v| v.as_global_value().as_pointer_value())
                    .collect::<Vec<_>>(),
            ),
        );

        // compile_constant
        let global = self.module.add_global(self.value_type(), None, &name);
        global.set_linkage(Linkage::Private);
        global.set_initializer(&self.value_type().const_zero());

        let function = self.module.get_function(&name).unwrap();
        self.set_current_definition(name.clone(), name, module.span);

        self.di.push_subprogram(function.get_subprogram().unwrap());
        self.di.push_block_scope(module.span);
        self.set_span(module.span);

        let basic_block = self.context.append_basic_block(function, "entry");
        let initialize = self.context.append_basic_block(function, "initialize");
        let initialized = self.context.append_basic_block(function, "initialized");
        self.builder.position_at_end(basic_block);

        self.branch_undefined(global.as_pointer_value(), initialize, initialized);

        self.builder.position_at_end(initialized);
        let sret = function.get_first_param().unwrap().into_pointer_value();
        self.trilogy_value_clone_into(sret, global.as_pointer_value());
        self.builder.build_return(None).unwrap();

        self.builder.position_at_end(initialize);

        self.trilogy_module_init_new(
            global.as_pointer_value(),
            self.context
                .i64_type()
                .const_int(members.len() as u64, false),
            member_ids_global.as_pointer_value(),
            members_global.as_pointer_value(),
            "",
        );

        self.builder
            .build_unconditional_branch(initialized)
            .unwrap();

        self.di.pop_scope();
        self.di.pop_scope();
    }

    fn import_module(&self, name: &Id, location: &str, module: &ir::Module) -> FunctionValue<'ctx> {
        for definition in module.definitions() {
            if !definition.is_exported {
                continue;
            }
            match &definition.item {
                DefinitionItem::Module(module) => {
                    self.declare_constructor(&format!("{location}::{}", module.name));
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
                DefinitionItem::Rule(..) => todo!(),
                DefinitionItem::Test(..) => continue,
            }
        }

        let constructor = self.declare_constructor(location);
        // The external module itself becomes aliased in this module:
        let accessor = self.module.add_function(
            &format!("{}::{name}", self.module_path()),
            self.accessor_type(),
            Some(Linkage::External),
        );
        accessor.set_call_conventions(LLVMCallConv::LLVMFastCallConv as u32);
        let sret = accessor.get_nth_param(0).unwrap().into_pointer_value();
        let accessor_entry = self.context.append_basic_block(accessor, "entry");
        self.builder.position_at_end(accessor_entry);
        self.call_internal(sret, constructor, &[]);
        self.builder.build_return(None).unwrap();
        accessor
    }

    fn declare_constructor(&self, location: &str) -> FunctionValue<'ctx> {
        if let Some(accessor) = self.module.get_function(location) {
            return accessor;
        }
        let accessor =
            self.module
                .add_function(location, self.accessor_type(), Some(Linkage::External));
        accessor.set_call_conventions(LLVMCallConv::LLVMFastCallConv as u32);
        accessor
    }
}
