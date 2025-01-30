use crate::{debug_info::DebugInfo, types};
use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    debug_info::AsDIScope,
    execution_engine::ExecutionEngine,
    llvm_sys::debuginfo::LLVMDIFlagPublic,
    memory_buffer::MemoryBuffer,
    module::Module,
    values::{BasicValue, FunctionValue, InstructionValue, PointerValue},
    AddressSpace, OptimizationLevel,
};
use source_span::Span;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::{Rc, Weak},
};
use trilogy_ir::{ir, Id};

#[derive(Clone)]
#[allow(dead_code)]
pub(crate) enum Head {
    Constant,
    Function,
    Procedure,
    Rule,
    Module(String),
}

#[must_use = "confirm that the current basic block will end without further instructions"]
pub(crate) struct NeverValue;

pub(crate) struct Codegen<'ctx> {
    pub(crate) atoms: Rc<RefCell<HashMap<String, u64>>>,
    pub(crate) context: &'ctx Context,
    pub(crate) module: Rc<Module<'ctx>>,
    pub(crate) builder: Builder<'ctx>,
    pub(crate) di: DebugInfo<'ctx>,
    pub(crate) execution_engine: ExecutionEngine<'ctx>,
    pub(crate) modules: &'ctx HashMap<String, &'ctx ir::Module>,
    pub(crate) globals: HashMap<Id, Head>,
    pub(crate) location: String,

    /// The chain of continuations arriving at the current expression being compiled.
    ///
    /// The last one is the current point. There should always be at least one (while
    /// compiling a function), and they should be topologically sorted according to their
    /// contained `capture_from` lists.
    pub(crate) continuation_points: RefCell<Vec<Rc<ContinuationPoint<'ctx>>>>,

    current_definition: RefCell<(String, Span)>,
}

/// During the reverse continuation phase, when closing this continuation block,
/// insert the instructions to build the closure after this instruction, and
/// replace this instruction with the allocation of a properly sized closure array.
#[derive(Default, Copy, Clone)]
enum Exit<'ctx> {
    #[default]
    Current,
    Continued {
        instruction: InstructionValue<'ctx>,
        block: BasicBlock<'ctx>,
    },
    Returned {
        instruction: InstructionValue<'ctx>,
    },
}

enum Variable<'ctx> {
    Closed(PointerValue<'ctx>),
    Owned(PointerValue<'ctx>),
}

impl<'ctx> Variable<'ctx> {
    fn ptr(&self) -> PointerValue<'ctx> {
        match self {
            Self::Closed(ptr) => *ptr,
            Self::Owned(ptr) => *ptr,
        }
    }
}

/// NOTE: Continuations for return, yield, and end are implicitly carried
/// as parameters to a continuation, as per calling convention.
#[derive(Default)]
pub(crate) struct ContinuationPoint<'ctx> {
    /// The end of this continuation point. Only one exit is necessary, as the continuation
    /// is split at any branch, one for each branch target, and gets merged afterwards.
    ///
    /// The latest continuation point does not have an exit set, as it has not yet exited.
    exit: Exit<'ctx>,

    /// Pointers to variables available at this point in the continuation.
    /// These pointers may be to values on stack, or to locations in the closure.
    variables: RefCell<HashMap<Id, Variable<'ctx>>>,
    /// The list of all variables which can possibly be referenced from this location.
    /// If the variable is not already referenced (i.e. found in the `variables` map),
    /// then it must be requested from all of the `capture_from` continuation points
    /// and added to the closure array and variables map.
    parent_variables: HashSet<Id>,

    /// Maintains the order of variables found in the closure array.
    closure: RefCell<Vec<Id>>,
    /// The mapping from variable names to their upvalues. If one already exists for a variable
    /// as it is being captured, it must be reused.
    upvalues: RefCell<HashMap<Id, PointerValue<'ctx>>>,
    /// The lexical pre-continuations from which this continuation may be reached. May be many
    /// in the case of branching instructions such as `if` or `match`.
    #[expect(dead_code, reason = "I feel like needed but maybe it's not")]
    capture_from: Vec<Weak<ContinuationPoint<'ctx>>>,
}

impl<'ctx> ContinuationPoint<'ctx> {
    fn child(parent: &Rc<ContinuationPoint<'ctx>>) -> Self {
        Self {
            exit: Exit::Current,
            variables: RefCell::default(),
            closure: RefCell::default(),
            parent_variables: parent
                .variables
                .borrow()
                .keys()
                .chain(parent.parent_variables.iter())
                .cloned()
                .collect(),
            upvalues: RefCell::default(),
            capture_from: vec![Rc::downgrade(parent)],
        }
    }
}

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn get_function(&self) -> FunctionValue<'ctx> {
        self.builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap()
    }

    pub(crate) fn get_return(&self) -> PointerValue<'ctx> {
        let container = self.allocate_value("");
        self.trilogy_value_clone_into(
            container,
            self.get_function()
                .get_nth_param(0)
                .unwrap()
                .into_pointer_value(),
        );
        container
    }

    pub(crate) fn get_yield(&self) -> PointerValue<'ctx> {
        let container = self.allocate_value("");
        self.trilogy_value_clone_into(
            container,
            self.get_function()
                .get_nth_param(1)
                .unwrap()
                .into_pointer_value(),
        );
        container
    }

    pub(crate) fn get_end(&self) -> PointerValue<'ctx> {
        let container = self.allocate_value("");
        self.trilogy_value_clone_into(
            container,
            self.get_function()
                .get_nth_param(2)
                .unwrap()
                .into_pointer_value(),
        );
        container
    }

    pub(crate) fn get_continuation(&self) -> PointerValue<'ctx> {
        let container = self.allocate_value("");
        self.trilogy_value_clone_into(
            container,
            self.get_function()
                .get_nth_param(3)
                .unwrap()
                .into_pointer_value(),
        );
        container
    }

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
        let di = DebugInfo::new(&module, "trilogy:runtime");

        let codegen = Codegen {
            atoms: Rc::new(RefCell::new(atoms)),
            builder: context.create_builder(),
            di,
            context,
            execution_engine: module
                .create_jit_execution_engine(OptimizationLevel::Default)
                .unwrap(),
            module: Rc::new(module),
            modules,
            globals: HashMap::default(),
            location: "trilogy:runtime".to_owned(),
            continuation_points: RefCell::default(),
            current_definition: RefCell::default(),
        };

        codegen
    }

    /// Creates a `Codegen` for another (distinct) Trilogy module.
    ///
    /// This function is called `sub` as in `submodule` incorrectly; it is not for creating
    /// a `Codegen` for a Trilogy module's submodule.
    pub(crate) fn sub(&self, name: &str) -> Codegen<'ctx> {
        let module = self.context.create_module(name);
        let di = DebugInfo::new(&module, name);
        Codegen {
            atoms: self.atoms.clone(),
            context: self.context,
            builder: self.context.create_builder(),
            di,
            execution_engine: self.execution_engine.clone(),
            module: Rc::new(module),
            modules: self.modules,
            globals: HashMap::new(),
            location: name.to_owned(),
            continuation_points: RefCell::default(),
            current_definition: RefCell::default(),
        }
    }

    /// Creates a `Codegen` for a sub-function.
    pub(crate) fn inner(&self) -> Codegen<'ctx> {
        let mut points = self.continuation_points.borrow().clone();
        points.push(Rc::new(ContinuationPoint::child(points.last().unwrap())));
        Codegen {
            atoms: self.atoms.clone(),
            context: self.context,
            builder: self.context.create_builder(),
            di: DebugInfo::new(&self.module, &self.location),
            execution_engine: self.execution_engine.clone(),
            module: self.module.clone(),
            modules: self.modules,
            globals: self.globals.clone(),
            location: self.location.clone(),
            continuation_points: RefCell::new(points),
            current_definition: RefCell::default(),
        }
    }

    pub(crate) fn set_current_definition(&self, name: String, span: Span) {
        *self.current_definition.borrow_mut() = (name, span);
        self.continuation_points
            .borrow_mut()
            .push(Rc::new(ContinuationPoint::default()));
    }

    pub(crate) fn get_current_definition(&self) -> (String, Span) {
        self.current_definition.borrow().clone()
    }

    pub(crate) fn set_returned(&self, instruction: InstructionValue<'ctx>) {
        Rc::get_mut(self.continuation_points.borrow_mut().last_mut().unwrap())
            .unwrap()
            .exit = Exit::Returned { instruction };
    }

    pub(crate) fn set_continued(&self, instruction: InstructionValue<'ctx>) {
        let mut cps = self.continuation_points.borrow_mut();
        Rc::get_mut(cps.last_mut().unwrap()).unwrap().exit = Exit::Continued {
            block: instruction.get_parent().unwrap(),
            instruction,
        };
        let new = Rc::new(ContinuationPoint::child(cps.last().unwrap()));
        cps.push(new);
    }

    fn clean_and_close_scope(&self, cp: &ContinuationPoint<'ctx>) {
        // TODO: clone debug scopes with new subprogram node
        for (id, var) in cp.variables.borrow().iter() {
            let Variable::Owned(pointer) = var else {
                continue;
            };

            if let Some(pointer) = cp.upvalues.borrow().get(id) {
                let upvalue = self.trilogy_reference_assume(*pointer);
                self.trilogy_reference_close(upvalue);
            } else {
                self.trilogy_value_destroy(*pointer);
            }
        }
        for param in self.get_function().get_param_iter() {
            self.trilogy_value_destroy(param.into_pointer_value());
        }
    }

    pub(crate) fn cleanup_scope(&self, cp: &ContinuationPoint<'ctx>) {
        for var in cp.variables.borrow().values() {
            let Variable::Owned(pointer) = var else {
                continue;
            };

            self.trilogy_value_destroy(*pointer);
        }
        for param in self.get_function().get_param_iter() {
            self.trilogy_value_destroy(param.into_pointer_value());
        }
    }

    pub(crate) fn close_continuation(&self) {
        let mut child: Option<Rc<ContinuationPoint<'ctx>>> = None;

        while let Some(parent) = {
            let mut rcs = self.continuation_points.borrow_mut();
            rcs.pop()
        } {
            // This is where we do the cleanup, and then call the return continuation if we haven't already
            // set up an exit.

            // TODO: ensure debug scope is accurate

            match parent.exit {
                Exit::Current => {
                    self.clean_and_close_scope(&parent);
                    // If the current lexical continuation ends without value, then it should `return unit`
                    //
                    // When we're in the current continuation, and we hit the end, then we need to fake
                    // a return. This might be best moved elsewhere
                    self.call_continuation(
                        self.get_function()
                            .get_first_param()
                            .unwrap()
                            .into_pointer_value(),
                        self.unit_const().into(),
                    );
                    self.builder.build_return(None).unwrap();
                }
                Exit::Returned { instruction, .. } => {
                    // If the current lexical continuation ended by returning, then we're just injecting
                    // cleanup before the call to the return continuation
                    self.builder.position_before(&instruction);
                    self.clean_and_close_scope(&parent);
                }
                Exit::Continued { instruction, block } => {
                    let child = child.unwrap();
                    self.builder.position_at(block, &instruction);
                    let closure = self.build_closure(&parent, &child);
                    self.clean_and_close_scope(&child);
                    instruction.replace_all_uses_with(&closure.as_instruction_value().unwrap());
                    instruction.erase_from_basic_block();
                }
            }

            child = Some(parent);
        }
    }

    pub(crate) fn close_as_do(
        &self,
        target: PointerValue<'ctx>,
        function: FunctionValue<'ctx>,
        arity: usize,
    ) {
        let mut rcs = self.continuation_points.borrow_mut();
        let child_scope = rcs.pop().unwrap();
        let scope = rcs.last().unwrap().clone();
        let closure = self.build_closure(&scope, &child_scope);
        self.trilogy_callable_init_do(
            target,
            arity,
            closure,
            function.as_global_value().as_pointer_value(),
        );
    }

    fn build_closure(
        &self,
        scope: &ContinuationPoint<'ctx>,
        child_scope: &ContinuationPoint<'ctx>,
    ) -> PointerValue<'ctx> {
        let closure_size = child_scope.closure.borrow().len();
        let closure = self.allocate_value("closure");
        let closure_array = self.trilogy_array_init_cap(closure, closure_size, "closure.payload");
        for id in child_scope.closure.borrow().iter() {
            let new_upvalue = self.allocate_value("");
            if let Some(ptr) = scope.upvalues.borrow().get(id) {
                self.trilogy_value_clone_into(new_upvalue, *ptr);
            } else if let Some(variable) = scope.variables.borrow().get(id) {
                self.trilogy_reference_to(new_upvalue, variable.ptr());
                scope.upvalues.borrow_mut().insert(id.clone(), new_upvalue);
            } else if scope.parent_variables.contains(id) {
                let variable = self.get_variable(id).expect("closure is messed up");
                self.trilogy_reference_to(new_upvalue, variable);
                scope.upvalues.borrow_mut().insert(id.clone(), new_upvalue);
            }
            self.trilogy_array_push(closure_array, new_upvalue);
        }
        closure
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
        (Rc::into_inner(self.module).unwrap(), self.execution_engine)
    }

    pub(crate) fn current_continuation(&self) -> Rc<ContinuationPoint<'ctx>> {
        self.continuation_points.borrow().last().unwrap().clone()
    }

    pub(crate) fn get_variable(&self, id: &Id) -> Option<PointerValue<'ctx>> {
        let scope = self.current_continuation();
        if let Some(var) = scope.variables.borrow().get(id) {
            return Some(var.ptr());
        }

        if scope.parent_variables.contains(id) {
            let mut closure = scope.closure.borrow_mut();
            let closure_index = closure.len();
            closure.push(id.clone());

            // Closure is an array of `trilogy_value`, where each of those values is a reference.
            // To access the variable:
            // 1. Consider the nth element of the array
            // 2. Get the value inside
            // 3. Assume a reference, and load its location field
            // 4. That value of that location field is the pointer to the actual value
            let closure_ptr = self
                .get_function()
                .get_last_param()
                .unwrap()
                .into_pointer_value();
            let location = unsafe {
                let array_entry = self
                    .builder
                    .build_gep(
                        self.value_type().array_type(0),
                        closure_ptr,
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
                .borrow_mut()
                .insert(id.clone(), Variable::Closed(location));
            return Some(location);
        }

        None
    }

    pub(crate) fn variable(&self, id: &ir::Identifier) -> PointerValue<'ctx> {
        if let Some(variable) = self.get_variable(&id.id) {
            return variable;
        }

        let builder = self.context.create_builder();
        let entry = self.get_function().get_first_basic_block().unwrap();
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
        let scope = self.current_continuation();
        scope
            .variables
            .borrow_mut()
            .insert(id.id.clone(), Variable::Owned(variable));

        if let Some(subp) = self.get_function().get_subprogram() {
            if let Some(name) = id.id.name() {
                let di_variable = self.di.builder.create_auto_variable(
                    subp.as_debug_info_scope(),
                    name,
                    self.di.unit.get_file(),
                    id.declaration_span.start().line as u32 + 1,
                    self.di.value_di_type().as_type(),
                    true,
                    LLVMDIFlagPublic,
                    0,
                );
                let di_location = self.di.builder.create_debug_location(
                    self.context,
                    id.span.start().line as u32 + 1,
                    id.span.start().column as u32 + 1,
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
