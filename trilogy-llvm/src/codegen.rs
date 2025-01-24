#![expect(dead_code, reason = "WIP")]

use crate::{
    debug_info::DebugInfo,
    scope::{Scope, Variable},
    types,
};
use inkwell::{
    attributes::{Attribute, AttributeLoc},
    builder::Builder,
    context::Context,
    debug_info::AsDIScope,
    execution_engine::ExecutionEngine,
    llvm_sys::debuginfo::LLVMDIFlagPublic,
    memory_buffer::MemoryBuffer,
    module::{Linkage, Module},
    values::{FunctionValue, PointerValue},
    AddressSpace, OptimizationLevel,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use trilogy_ir::{ir, Id};

pub(crate) enum Head {
    Constant,
    Function,
    Procedure(usize),
    Rule(usize),
    Module(String),
}

#[must_use = "confirm that the current basic block will end without further instructions"]
pub(crate) struct NeverValue;

pub(crate) struct Codegen<'ctx> {
    pub(crate) atoms: Rc<RefCell<HashMap<String, u64>>>,
    pub(crate) context: &'ctx Context,
    pub(crate) module: Module<'ctx>,
    pub(crate) builder: Builder<'ctx>,
    pub(crate) di: DebugInfo<'ctx>,
    pub(crate) execution_engine: ExecutionEngine<'ctx>,
    pub(crate) modules: &'ctx HashMap<String, &'ctx ir::Module>,
    pub(crate) globals: HashMap<Id, Head>,
    pub(crate) location: String,
}

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn new(
        context: &'ctx Context,
        modules: &'ctx HashMap<String, &'ctx ir::Module>,
    ) -> Self {
        let mut atoms = HashMap::new();
        atoms.insert("undefined".to_owned(), types::TAG_UNDEFINED);
        atoms.insert("unit".to_owned(), types::TAG_UNIT);
        atoms.insert("bool".to_owned(), types::TAG_BOOL);
        atoms.insert("atom".to_owned(), types::TAG_ATOM);
        atoms.insert("char".to_owned(), types::TAG_CHAR);
        atoms.insert("string".to_owned(), types::TAG_STRING);
        atoms.insert("number".to_owned(), types::TAG_NUMBER);
        atoms.insert("bits".to_owned(), types::TAG_BITS);
        atoms.insert("struct".to_owned(), types::TAG_STRUCT);
        atoms.insert("tuple".to_owned(), types::TAG_TUPLE);
        atoms.insert("array".to_owned(), types::TAG_ARRAY);
        atoms.insert("set".to_owned(), types::TAG_SET);
        atoms.insert("record".to_owned(), types::TAG_RECORD);
        atoms.insert("callable".to_owned(), types::TAG_CALLABLE);

        let module = context.create_module("trilogy:runtime");
        let di = DebugInfo::new(&module, "trilogy:runtime", ".");

        let codegen = Codegen {
            atoms: Rc::new(RefCell::new(atoms)),
            builder: context.create_builder(),
            di,
            context,
            execution_engine: module
                .create_jit_execution_engine(OptimizationLevel::Default)
                .unwrap(),
            module,
            modules,
            globals: HashMap::default(),
            location: "trilogy:runtime".to_owned(),
        };

        codegen
    }

    pub(crate) fn allocate_value(&self, name: &str) -> PointerValue<'ctx> {
        let value = self.builder.build_alloca(self.value_type(), name).unwrap();
        self.builder
            .build_store(value, self.value_type().const_zero())
            .unwrap();
        value
    }

    fn build_atom_registry(&self) {
        let atoms = self.atoms.borrow();
        let mut atoms_vec: Vec<_> = atoms.iter().collect();
        atoms_vec.sort_by_key(|(_, s)| **s);
        let atom_registry_sz =
            self.module
                .add_global(self.context.i64_type(), None, "atom_registry_sz");
        atom_registry_sz.set_initializer(
            &self
                .context
                .i64_type()
                .const_int(atoms_vec.len() as u64, false),
        );
        let atom_registry = self.module.add_global(
            self.string_value_type().array_type(atoms_vec.len() as u32),
            None,
            "atom_registry",
        );
        let atom_table: Vec<_> = atoms_vec
            .into_iter()
            .map(|(atom, _)| {
                let bytes = atom.as_bytes();
                let string = self.module.add_global(
                    self.context.i8_type().array_type(bytes.len() as u32),
                    None,
                    "",
                );
                string.set_initializer(&self.context.const_string(bytes, false));
                self.string_value_type().const_named_struct(&[
                    self.context
                        .i64_type()
                        .const_int(bytes.len() as u64, false)
                        .into(),
                    string.as_pointer_value().into(),
                ])
            })
            .collect();
        atom_registry.set_initializer(&self.string_value_type().const_array(&atom_table));
    }

    pub(crate) fn finish(self) -> (Module<'ctx>, ExecutionEngine<'ctx>) {
        self.build_atom_registry();

        let core =
            MemoryBuffer::create_from_memory_range(include_bytes!("../core/core.bc"), "core");
        let core = Module::parse_bitcode_from_buffer(&core, self.context).unwrap();
        self.module.link_in_module(core).unwrap();
        self.di.builder.finalize();
        (self.module, self.execution_engine)
    }

    pub(crate) fn add_function(&self, name: &str, exported: bool) -> FunctionValue<'ctx> {
        let procedure = self.module.add_function(
            name,
            self.procedure_type(1, false),
            if exported {
                Some(Linkage::External)
            } else {
                Some(Linkage::Private)
            },
        );
        procedure.add_attribute(
            AttributeLoc::Param(0),
            self.context.create_type_attribute(
                Attribute::get_named_enum_kind_id("sret"),
                self.value_type().into(),
            ),
        );
        procedure.get_nth_param(0).unwrap().set_name("sretptr");
        procedure
    }

    pub(crate) fn get_variable(
        &self,
        scope: &mut Scope<'ctx>,
        id: &ir::Identifier,
    ) -> Option<PointerValue<'ctx>> {
        if scope.variables.contains_key(&id.id) {
            return Some(scope.variables.get(&id.id).unwrap().ptr());
        }

        if scope.parent_variables.contains(&id.id) {
            let closure_index = scope.closure.len();
            scope.closure.push(id.id.clone());

            // Closure is an array of trilogy values, where each of those trilogy values is a reference
            // To access the variable then:
            // 1. Consider the nth element of the array
            // 2. Get the value inside
            // 3. Assume a reference, and load its location field
            // 4. That value of that location field is the pointer to the actual value
            let closure = scope.get_closure_ptr();
            let location = unsafe {
                let array_entry = self
                    .builder
                    .build_gep(
                        self.value_type().array_type(0),
                        closure,
                        &[
                            self.context.i32_type().const_int(0, false),
                            self.context
                                .i32_type()
                                .const_int(closure_index as u64, false),
                        ],
                        "",
                    )
                    .unwrap();
                let ref_value = self.trilogy_reference_assume(array_entry);
                let location = self
                    .builder
                    .build_struct_gep(self.reference_value_type(), ref_value, 1, "")
                    .unwrap();
                self.builder
                    .build_load(self.context.ptr_type(AddressSpace::default()), location, "")
                    .unwrap()
                    .into_pointer_value()
            };
            scope
                .variables
                .insert(id.id.clone(), Variable::Closed(location));
            return Some(location);
        }

        None
    }

    pub(crate) fn variable(
        &self,
        scope: &mut Scope<'ctx>,
        id: &ir::Identifier,
    ) -> PointerValue<'ctx> {
        if let Some(variable) = self.get_variable(scope, id) {
            return variable;
        }
        let builder = self.context.create_builder();
        let entry = scope.function.get_first_basic_block().unwrap();
        match entry.get_first_instruction() {
            Some(instruction) => builder.position_before(&instruction),
            None => builder.position_at_end(entry),
        }

        let variable = builder
            .build_alloca(self.value_type(), &id.to_string())
            .unwrap();
        builder
            .build_store(variable, self.value_type().const_zero())
            .unwrap();
        scope
            .variables
            .insert(id.id.clone(), Variable::Owned(variable));

        if let Some(subp) = scope.function.get_subprogram() {
            if let Some(name) = id.id.name() {
                let di_variable = self.di.builder.create_auto_variable(
                    subp.as_debug_info_scope(),
                    name,
                    self.di.unit.get_file(),
                    id.declaration_span.start().line as u32,
                    self.di.value_di_type().as_type(),
                    true,
                    LLVMDIFlagPublic,
                    0,
                );
                let di_location = self.di.builder.create_debug_location(
                    self.context,
                    id.span.start().line as u32,
                    id.span.start().column as u32,
                    subp.as_debug_info_scope(),
                    None,
                );
                self.di.builder.insert_declare_at_end(
                    variable,
                    Some(di_variable),
                    None,
                    di_location,
                    builder.get_insert_block().unwrap(),
                );
            }
        }

        variable
    }

    pub(crate) fn embed_c_string<S: AsRef<str>>(&self, string: S) -> PointerValue<'ctx> {
        let string = string.as_ref();
        let global = self.module.add_global(
            self.context.i8_type().array_type((string.len() + 1) as u32),
            None,
            "",
        );
        global.set_initializer(&self.context.const_string(string.as_bytes(), true));
        global.as_pointer_value()
    }
}
