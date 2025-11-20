use crate::codegen::{Codegen, Global, Head, Variable};
use inkwell::AddressSpace;
use inkwell::debug_info::AsDIScope;
use inkwell::llvm_sys::debuginfo::LLVMDIFlagPublic;
use inkwell::module::Linkage;
use inkwell::values::{BasicValue, FunctionValue, PointerValue};
use std::collections::BTreeMap;
use trilogy_ir::Id;
use trilogy_ir::ir::{self, DefinitionItem};

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn compile_module(&self, file: &str, module: &ir::Module) -> Codegen<'ctx> {
        let mut subcontext = self.for_file(file);
        subcontext.compile_module_contents(module, None, true);
        subcontext
    }

    fn add_global(&mut self, id: Id, head: Head) {
        self.globals.insert(
            id.clone(),
            Global {
                path: self.path.clone(),
                id,
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
        &mut self,
        module: &ir::Module,
        mut module_context: Option<Vec<Id>>,
        is_public: bool,
    ) {
        let previous_module_context = module_context.clone();
        let constructor_name = self.module_path();
        let constructor_accessor = self.declare_constructor(
            &constructor_name,
            previous_module_context.is_some(),
            is_public,
        );

        if !module.parameters.is_empty() {
            let context = module_context.get_or_insert_default();
            context.extend(module.parameters.iter().map(|id| id.id.clone()));
        }

        // Pre-declare everything this module will reference so that all references during codegen will
        // be valid.
        let mut members = BTreeMap::new();
        for definition in module.definitions() {
            let linkage = if definition.is_exported {
                Linkage::External
            } else {
                Linkage::Private
            };
            let member_accessor = match &definition.item {
                DefinitionItem::Module(module) if module.module.as_external().is_some() => {
                    // TODO: probably most sensible to just disallow external modules to be imported in functor
                    // modules since they won't be able to access the context values anyway.
                    let location = module.module.as_external().unwrap().to_owned();
                    let submodule = self.modules.get(&location).unwrap();
                    self.add_global(
                        module.name.id.clone(),
                        Head::ExternalModule(location.clone()),
                    );
                    self.import_module(definition.name().unwrap(), &location, submodule)
                }
                DefinitionItem::Module(def) => {
                    let module = def.module.as_module().unwrap();
                    let head_type = if module.parameters.is_empty() {
                        if let Some(context) = &mut module_context {
                            context.push(def.name.id.clone());
                        }
                        Head::Module
                    } else {
                        Head::Function
                    };
                    self.add_global(def.name.id.clone(), head_type);
                    let accessor_name = format!("{}::{}", self.module_path(), def.name);
                    self.add_accessor(&accessor_name, module_context.is_some(), linkage)
                }
                DefinitionItem::Constant(constant) => {
                    self.add_global(constant.name.id.clone(), Head::Constant);
                    if let Some(context) = &mut module_context {
                        context.push(constant.name.id.clone());
                    }
                    let accessor_name = format!("{}::{}", self.module_path(), constant.name);
                    self.add_accessor(&accessor_name, module_context.is_some(), linkage)
                }
                DefinitionItem::Procedure(procedure) if procedure.overloads.is_empty() => {
                    // TODO: probably most sensible to just disallow extern procedures to be defined in functor modules
                    // since they won't be able to access the context values anyway.
                    self.add_global(procedure.name.id.clone(), Head::Procedure);
                    self.declare_extern_procedure(
                        &procedure.name.to_string(),
                        procedure.arity,
                        linkage,
                        procedure.span(),
                    )
                }
                DefinitionItem::Procedure(procedure) => {
                    self.add_global(procedure.name.id.clone(), Head::Procedure);
                    let accessor_name = format!("{}::{}", self.module_path(), procedure.name);
                    self.add_accessor(&accessor_name, module_context.is_some(), linkage)
                }
                DefinitionItem::Function(function) => {
                    self.add_global(function.name.id.clone(), Head::Function);
                    let accessor_name = format!("{}::{}", self.module_path(), function.name);
                    self.add_accessor(&accessor_name, module_context.is_some(), linkage)
                }
                DefinitionItem::Rule(rule) => {
                    self.add_global(rule.name.id.clone(), Head::Rule);
                    let accessor_name = format!("{}::{}", self.module_path(), rule.name);
                    self.add_accessor(&accessor_name, module_context.is_some(), linkage)
                }
                DefinitionItem::Test(..) => continue,
            };
            let member_id = self.atom_value_raw(definition.name().unwrap().to_string());
            members.insert(member_id, (definition.is_exported, member_accessor));
        }

        // With all those pre-declared, build the constructor for this module.
        let module_data_global = self.build_module_data(members);
        if module.parameters.is_empty() {
            // A module with no parameters comes across as a constant, and returns
            // a module object with no closure.
            self.compile_constant_constructor(
                constructor_accessor,
                module,
                module_data_global,
                is_public,
                previous_module_context,
                module_context.clone(),
            );
        } else {
            // A module with parameters looks like a function, and returns a module
            // object with a closure. The module access operator pulls values from
            // that closure to build the function members. Function equivalence in
            // this situation will be tricky...
            self.compile_functor_constructor(
                constructor_accessor,
                module,
                module_data_global,
                is_public,
                previous_module_context,
                module_context.clone().unwrap(),
            );
        }

        // Then comes actual codegen
        for definition in module.definitions() {
            match &definition.item {
                // An extern procedure is just an accessor, so there is no code to generate
                DefinitionItem::Procedure(procedure) if procedure.overloads.is_empty() => {}
                // External modules are also just accessors for all the top level public members
                // with no additional code
                DefinitionItem::Module(module) if module.module.as_external().is_some() => {}
                DefinitionItem::Module(def) => {
                    let module = def.module.as_module().unwrap();
                    self.begin_submodule(def.name.to_string());
                    self.compile_module_contents(
                        module,
                        module_context.clone(),
                        definition.is_exported,
                    );
                    self.end_submodule();
                }
                DefinitionItem::Procedure(procedure) => {
                    self.compile_procedure(procedure, module_context.clone());
                }
                DefinitionItem::Function(function) => {
                    self.compile_function(function, module_context.clone());
                }
                DefinitionItem::Constant(constant) => {
                    self.compile_constant(constant, module_context.clone());
                }
                DefinitionItem::Rule(rule) => self.compile_rule(rule, module_context.clone()),
                DefinitionItem::Test(..) => {}
            }
        }
    }

    fn build_module_data(
        &self,
        members: BTreeMap<u64, (bool, FunctionValue<'ctx>)>,
    ) -> PointerValue<'ctx> {
        // The members are all global arrays, which are then assembled into the module_data structure
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

        let members_byte_len = members.len() / 8 + if members.len() & 7 != 0 { 1 } else { 0 };
        let members_exported_global = self.module.add_global(
            self.context.i8_type().array_type(members_byte_len as u32),
            None,
            "",
        );
        let mut members_exported = vec![0; members_byte_len];
        for (i, (is_exported, _)) in members.values().enumerate() {
            if *is_exported {
                let byte_index = i / 8;
                let bit_index = i % 8;
                members_exported[byte_index] |= 1 << bit_index;
            }
        }
        members_exported_global.set_initializer(
            &self.context.i8_type().const_array(
                &members_exported
                    .into_iter()
                    .map(|v| self.context.i8_type().const_int(v, false))
                    .collect::<Vec<_>>(),
            ),
        );

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
                    .map(|(_, v)| v.as_global_value().as_pointer_value())
                    .collect::<Vec<_>>(),
            ),
        );

        let module_data_type = self.context.struct_type(
            &[
                self.context
                    .ptr_sized_int_type(self.execution_engine.get_target_data(), None)
                    .into(),
                self.context.ptr_type(AddressSpace::default()).into(),
                self.context.ptr_type(AddressSpace::default()).into(),
                self.context.ptr_type(AddressSpace::default()).into(),
            ],
            false,
        );
        let module_data_global = self.module.add_global(module_data_type, None, "");
        module_data_global.set_initializer(
            &module_data_type.const_named_struct(&[
                self.context
                    .ptr_sized_int_type(self.execution_engine.get_target_data(), None)
                    .const_int(members.len() as u64, false)
                    .into(),
                member_ids_global.as_pointer_value().into(),
                members_exported_global.as_pointer_value().into(),
                members_global.as_pointer_value().into(),
            ]),
        );
        module_data_global.as_pointer_value()
    }

    fn compile_constant_constructor(
        &self,
        accessor: FunctionValue<'ctx>,
        module: &ir::Module,
        module_data_global: PointerValue<'ctx>,
        is_public: bool,
        previous_module_context: Option<Vec<Id>>,
        module_context: Option<Vec<Id>>,
    ) {
        let name = self.module_path();
        let subprogram = self.di.builder.create_function(
            self.di.unit.as_debug_info_scope(),
            &name,
            None,
            self.di.unit.get_file(),
            module.span.start().line as u32 + 1,
            self.di.procedure_di_type(0),
            !is_public,
            true,
            module.span.start().line as u32 + 1,
            LLVMDIFlagPublic,
            false,
        );
        accessor.set_subprogram(subprogram);

        self.set_current_definition(
            name.clone(),
            name.clone(),
            module.span,
            previous_module_context,
        );
        self.begin_constant(accessor, module.span);
        let initialize = self.context.append_basic_block(accessor, "initialize");
        let initialized = self.context.append_basic_block(accessor, "initialized");

        // compile_constant
        let storage = if module_context.is_some() {
            self.allocate_value("")
        } else {
            let global = self.module.add_global(self.value_type(), None, &name);
            global.set_linkage(Linkage::Private);
            global.set_initializer(&self.value_type().const_zero());
            global.as_pointer_value()
        };

        self.branch_undefined(storage, initialize, initialized);

        self.builder.position_at_end(initialized);
        let sret = accessor.get_first_param().unwrap().into_pointer_value();
        self.trilogy_value_clone_into(sret, storage);
        self.builder.build_return(None).unwrap();

        self.builder.position_at_end(initialize);

        if let Some(module_context) = module_context {
            let closure = self.allocate_value("");
            let array = self.trilogy_array_init_cap(closure, module_context.len(), "");
            let mut references = vec![];
            for id in &module_context {
                let upvalue = match self.get_variable(id) {
                    Some(Variable::Closed { upvalue, .. }) => {
                        let new_upvalue = self.allocate_value(&format!("{id}.reup"));
                        self.trilogy_value_clone_into(new_upvalue, upvalue);
                        new_upvalue
                    }
                    Some(Variable::Owned(variable)) => {
                        let upvalue = self.allocate_value(&format!("{id}.up"));
                        let reference =
                            self.trilogy_reference_to(upvalue, variable, &format!("*{id}.up"));
                        self.trilogy_reference_close(reference);
                        upvalue
                    }
                    None => {
                        let upvalue = self.allocate_value("");
                        references.push((self.trilogy_reference_init_empty(upvalue, ""), id));
                        upvalue
                    }
                };
                self.trilogy_array_push(array, upvalue);
            }

            for (reference, id) in references {
                let global = self.globals.get(id).unwrap();
                let location = self.trilogy_reference_get_location(reference, "");
                self.init_global(location, global, closure);
            }

            self.trilogy_module_init_new_closure(storage, module_data_global, closure, "");
        } else {
            self.trilogy_module_init_new(storage, module_data_global, "");
        }

        self.builder
            .build_unconditional_branch(initialized)
            .unwrap();

        self.end_function();
    }

    fn compile_functor_constructor(
        &self,
        accessor: FunctionValue<'ctx>,
        module: &ir::Module,
        module_data_global: PointerValue<'ctx>,
        is_public: bool,
        previous_module_context: Option<Vec<Id>>,
        module_context: Vec<Id>,
    ) {
        let has_context = accessor.count_params() == 2;

        // declare_function
        let name = self.module_path();
        let constructor_name = format!("{name}:::constructor");
        let function = self.add_function(
            &constructor_name,
            &name,
            module.span,
            previous_module_context.is_some(),
            !is_public,
        );

        // write_function_accessor
        let sret = accessor.get_nth_param(0).unwrap().into_pointer_value();
        let accessor_entry = self.context.append_basic_block(accessor, "entry");
        self.builder.position_at_end(accessor_entry);
        if has_context {
            let context = accessor.get_nth_param(1).unwrap().into_pointer_value();
            self.trilogy_callable_init_fn(sret, context, function);
        } else {
            let context = self.allocate_value("");
            self.trilogy_array_init_cap(context, 0, "");
            self.trilogy_callable_init_fn(sret, context, function);
        }
        self.builder.build_return(None).unwrap();

        // compile_function_body
        self.set_current_definition(name.clone(), name, module.span, previous_module_context);
        self.begin_function(function, module.span);

        let arity = module.parameters.len();
        for i in 0..arity - 1 {
            let continuation = self.add_continuation("");
            let param = self.get_continuation("");
            let id = &module.parameters[i];
            let variable = self.variable(&id.id);
            self.trilogy_value_clone_into(variable, param);

            let return_to = self.get_return("");
            let cont_val = self.allocate_value("");
            let closure = self
                .builder
                .build_alloca(self.value_type(), "TEMP_CLOSURE")
                .unwrap();
            self.trilogy_callable_init_fn(cont_val, closure, continuation);
            let inner_cp = self.capture_contination_point(closure.as_instruction_value().unwrap());
            self.call_known_continuation(return_to, cont_val);

            self.become_continuation_point(inner_cp);
            self.begin_next_function(continuation);
        }

        // The last parameter is collected in the same continuation as the body
        let param = self.get_continuation("");
        let id = module.parameters.last().unwrap();
        let variable = self.variable(&id.id);
        self.trilogy_value_clone_into(variable, param);

        let closure = self.allocate_value("");
        let array = self.trilogy_array_init_cap(closure, module_context.len(), "");

        let mut references = vec![];
        for id in &module_context {
            let upvalue = match self.get_variable(id) {
                Some(Variable::Closed { upvalue, .. }) => {
                    let new_upvalue = self.allocate_value(&format!("{id}.reup"));
                    self.trilogy_value_clone_into(new_upvalue, upvalue);
                    new_upvalue
                }
                Some(Variable::Owned(variable)) => {
                    let upvalue = self.allocate_value(&format!("{id}.up"));
                    let reference =
                        self.trilogy_reference_to(upvalue, variable, &format!("*{id}.up"));
                    self.trilogy_reference_close(reference);
                    upvalue
                }
                None => {
                    let upvalue = self.allocate_value("");
                    references.push((self.trilogy_reference_init_empty(upvalue, ""), id));
                    upvalue
                }
            };
            self.trilogy_array_push(array, upvalue);
        }

        for (reference, id) in references {
            let global = self.globals.get(id).unwrap();
            let location = self.trilogy_reference_get_location(reference, "");
            self.init_global(location, global, closure);
        }

        let target = self.allocate_value("");
        self.trilogy_module_init_new_closure(target, module_data_global, closure, "");
        let ret = self.get_return("");
        self.call_known_continuation(ret, target);
        self.end_function();
    }

    fn init_global(
        &self,
        target: PointerValue<'ctx>,
        global: &Global,
        closure: PointerValue<'ctx>,
    ) {
        let global_name = format!("{}::{}", global.module_path(&self.location), global.id);
        let function = self
            .module
            .get_function(&global_name)
            .expect("function was not defined");
        self.call_internal(target, function, &[closure.into()]);
    }

    fn import_module(&self, name: &Id, location: &str, module: &ir::Module) -> FunctionValue<'ctx> {
        for definition in module.definitions() {
            if !definition.is_exported {
                continue;
            }
            match &definition.item {
                DefinitionItem::Module(module) => {
                    let is_function_module = module
                        .module
                        .as_module()
                        .is_some_and(|module| !module.parameters.is_empty());
                    self.declare_constructor(
                        &format!("{location}::{}", module.name),
                        is_function_module,
                        true,
                    );
                }
                DefinitionItem::Procedure(procedure) => {
                    self.import_accessor(&format!("{}::{}", location, &procedure.name.to_string()));
                }
                DefinitionItem::Function(function) => {
                    self.import_accessor(&format!("{}::{}", location, &function.name.to_string()));
                }
                DefinitionItem::Constant(constant) => {
                    self.import_accessor(&format!("{}::{}", location, &constant.name.to_string()));
                }
                DefinitionItem::Rule(rule) => {
                    self.import_accessor(&format!("{}::{}", location, &rule.name.to_string()));
                }
                DefinitionItem::Test(..) => unreachable!(),
            }
        }

        let constructor = self.declare_constructor(location, false, true);
        // The external module itself becomes aliased in this module:
        let alias_name = format!("{}::{}", self.module_path(), name);
        let accessor = self.add_accessor(&alias_name, false, Linkage::External);
        let sret = accessor.get_nth_param(0).unwrap().into_pointer_value();
        let accessor_entry = self.context.append_basic_block(accessor, "entry");
        self.builder.position_at_end(accessor_entry);
        self.call_internal(sret, constructor, &[]);
        self.builder.build_return(None).unwrap();
        accessor
    }

    fn declare_constructor(
        &self,
        location: &str,
        has_context: bool,
        is_public: bool,
    ) -> FunctionValue<'ctx> {
        if let Some(accessor) = self.module.get_function(location) {
            return accessor;
        }
        self.add_accessor(
            location,
            has_context,
            if is_public {
                Linkage::External
            } else {
                Linkage::Private
            },
        )
    }
}
