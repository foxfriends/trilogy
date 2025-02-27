use crate::{debug_info::DebugInfo, types};
use inkwell::{
    AddressSpace, OptimizationLevel,
    builder::Builder,
    context::Context,
    debug_info::{AsDIScope, DILocation},
    execution_engine::ExecutionEngine,
    llvm_sys::debuginfo::LLVMDIFlagPublic,
    module::Module,
    values::{BasicValue, FunctionValue, InstructionValue, PointerValue},
};
use source_span::Span;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::{Rc, Weak},
};
use trilogy_ir::{Id, ir};

#[derive(Clone)]
#[expect(dead_code)]
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
    continuation_points: RefCell<Vec<Rc<ContinuationPoint<'ctx>>>>,

    current_definition: RefCell<(String, Span)>,
}

#[derive(Clone, Debug)]
struct Parent<'ctx> {
    parent: Weak<ContinuationPoint<'ctx>>,
    instruction: InstructionValue<'ctx>,
    debug_location: DILocation<'ctx>,
}

/// During the reverse continuation phase, when closing this continuation block,
/// insert the instructions to build the closure after this instruction, and
/// replace this instruction with the allocation of a properly sized closure array.
#[derive(Clone, Debug)]
enum Exit<'ctx> {
    Close(Parent<'ctx>),
    Clean(Parent<'ctx>),
    Capture(Parent<'ctx>),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Variable<'ctx> {
    Closed {
        location: PointerValue<'ctx>,
        upvalue: PointerValue<'ctx>,
    },
    Owned(PointerValue<'ctx>),
}

impl<'ctx> Variable<'ctx> {
    pub(crate) fn ptr(&self) -> PointerValue<'ctx> {
        match self {
            Self::Closed { location, .. } => *location,
            Self::Owned(ptr) => *ptr,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
enum Closed<'ctx> {
    Variable(Id),
    Temporary(PointerValue<'ctx>),
}

impl std::fmt::Display for Closed<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Variable(id) => id.fmt(f),
            Self::Temporary(ptr) => write!(f, "{}.ref", ptr.get_name().to_str().unwrap()),
        }
    }
}

/// NOTE: Continuations for return, yield, and end are implicitly carried
/// as parameters to a continuation, as per calling convention.
#[derive(Default, Debug)]
struct ContinuationPoint<'ctx> {
    /// Pointers to variables available at this point in the continuation.
    /// These pointers may be to values on stack, or to locations in the closure.
    variables: RefCell<HashMap<Closed<'ctx>, Variable<'ctx>>>,
    /// The list of all variables which can possibly be referenced from this location.
    /// If the variable is not already referenced (i.e. found in the `variables` map),
    /// then it must be requested from all of the `capture_from` continuation points
    /// and added to the closure array and variables map.
    parent_variables: HashSet<Closed<'ctx>>,

    /// Maintains the order of variables found in the closure array.
    closure: RefCell<Vec<Closed<'ctx>>>,
    /// The mapping from variable names to their upvalues. If one already exists for a variable
    /// as it is being captured, it must be reused.
    upvalues: RefCell<HashMap<Closed<'ctx>, PointerValue<'ctx>>>,
    /// The lexical pre-continuations from which this continuation may be reached. May be many
    /// in the case of branching instructions such as `if` or `match`.
    parents: Vec<Exit<'ctx>>,

    /// A bit of a hack, but this is tracking all places that a variable is destroyed during
    /// scope cleanup without being closed. If we later determine that we need to close that
    /// variable, this allows us to go back and make sure it was closed after all.
    unclosed: RefCell<HashMap<PointerValue<'ctx>, Vec<InstructionValue<'ctx>>>>,
}

impl<'ctx> ContinuationPoint<'ctx> {
    fn chain(&self) -> Self {
        Self {
            variables: RefCell::default(),
            closure: RefCell::default(),
            parent_variables: self
                .variables
                .borrow()
                .keys()
                .chain(self.parent_variables.iter())
                .cloned()
                .collect(),
            upvalues: RefCell::default(),
            parents: vec![],
            unclosed: RefCell::default(),
        }
    }

    fn close_from(
        &mut self,
        parent: &Rc<ContinuationPoint<'ctx>>,
        instruction: InstructionValue<'ctx>,
        debug_location: DILocation<'ctx>,
    ) {
        self.parents.push(Exit::Close(Parent {
            parent: Rc::downgrade(parent),
            instruction,
            debug_location,
        }));
    }

    fn clean_from(
        &mut self,
        parent: &Rc<ContinuationPoint<'ctx>>,
        instruction: InstructionValue<'ctx>,
        debug_location: DILocation<'ctx>,
    ) {
        self.parents.push(Exit::Clean(Parent {
            parent: Rc::downgrade(parent),
            instruction,
            debug_location,
        }));
    }

    fn capture_from(
        &mut self,
        parent: &Rc<ContinuationPoint<'ctx>>,
        instruction: InstructionValue<'ctx>,
        debug_location: DILocation<'ctx>,
    ) {
        self.parents.push(Exit::Capture(Parent {
            parent: Rc::downgrade(parent),
            instruction,
            debug_location,
        }));
    }
}

pub(crate) struct Brancher<'ctx>(Rc<ContinuationPoint<'ctx>>);
pub(crate) struct Merger<'ctx>(Vec<Exit<'ctx>>);

impl<'ctx> Merger<'ctx> {
    fn close_from(
        &mut self,
        parent: &Rc<ContinuationPoint<'ctx>>,
        instruction: InstructionValue<'ctx>,
        debug_location: DILocation<'ctx>,
    ) {
        self.0.push(Exit::Close(Parent {
            parent: Rc::downgrade(parent),
            instruction,
            debug_location,
        }));
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

    pub(crate) fn get_return(&self, name: &str) -> PointerValue<'ctx> {
        let container = self.allocate_value(name);
        let temp = self.builder.build_alloca(self.value_type(), "").unwrap();
        self.builder
            .build_store(temp, self.get_function().get_nth_param(0).unwrap())
            .unwrap();
        self.trilogy_value_clone_into(container, temp);
        container
    }

    pub(crate) fn get_yield(&self, name: &str) -> PointerValue<'ctx> {
        let container = self.allocate_value(name);
        let temp = self.builder.build_alloca(self.value_type(), "").unwrap();
        self.builder
            .build_store(temp, self.get_function().get_nth_param(1).unwrap())
            .unwrap();
        self.trilogy_value_clone_into(container, temp);
        container
    }

    pub(crate) fn get_end(&self, name: &str) -> PointerValue<'ctx> {
        let container = self.allocate_value(name);
        let temp = self.builder.build_alloca(self.value_type(), "").unwrap();
        self.builder
            .build_store(temp, self.get_function().get_nth_param(2).unwrap())
            .unwrap();
        self.trilogy_value_clone_into(container, temp);
        container
    }

    pub(crate) fn get_continuation(&self, name: &str) -> PointerValue<'ctx> {
        let container = self.allocate_value(name);
        let temp = self.builder.build_alloca(self.value_type(), "").unwrap();
        self.builder
            .build_store(temp, self.get_function().get_nth_param(3).unwrap())
            .unwrap();
        self.trilogy_value_clone_into(container, temp);
        container
    }

    fn get_closure_upvalue(
        &self,
        builder: &Builder<'ctx>,
        index: usize,
        name: &str,
    ) -> PointerValue<'ctx> {
        // Closure is passed as the last parameter, and is a Trilogy array of Trilogy reference
        // To access the variable:
        // 1. Consider the nth element of the array
        // 2. Get the value inside
        // 3. Assume a reference, and load its location field
        // 4. That value of that location field is the pointer to the actual value
        let closure_ptr = builder.build_alloca(self.value_type(), "").unwrap();
        builder
            .build_store(closure_ptr, self.get_function().get_last_param().unwrap())
            .unwrap();
        let closure_array = self.trilogy_array_assume_in(builder, closure_ptr);
        let upvalue = builder.build_alloca(self.value_type(), name).unwrap();
        builder
            .build_store(upvalue, self.value_type().const_zero())
            .unwrap();
        self.trilogy_array_at_in(
            builder,
            upvalue,
            closure_array,
            self.context.i64_type().const_int(index as u64, false),
        );
        upvalue
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
        atoms.insert("left".to_owned(), 14);
        atoms.insert("right".to_owned(), 15);
        atoms.insert("lt".to_owned(), 16);
        atoms.insert("eq".to_owned(), 17);
        atoms.insert("gt".to_owned(), 18);

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

    pub(crate) fn set_current_definition(&self, name: String, span: Span) {
        *self.current_definition.borrow_mut() = (name, span);
        self.continuation_points
            .borrow_mut()
            .push(Rc::new(ContinuationPoint::default()));
    }

    pub(crate) fn get_current_definition(&self) -> (String, Span) {
        self.current_definition.borrow().clone()
    }

    /// Ends the current continuation point. Cleanup code will be inserted before the provided
    /// instruction which captures any values referenced by the continuation, and destroys any
    /// remaining values that are going out of scope. The instruction should be an `alloca`
    /// used as if it were the closure value, and will be replaced with the actual closure
    /// construction instructions at a later time.
    pub(crate) fn end_continuation_point_as_close(
        &self,
        closure_allocation: InstructionValue<'ctx>,
        debug_location: DILocation<'ctx>,
    ) {
        let mut cps = self.continuation_points.borrow_mut();
        let last = cps.last().unwrap();
        let mut next = last.chain();
        next.close_from(last, closure_allocation, debug_location);
        cps.push(Rc::new(next));
    }

    /// Ends the current continuation point. Cleanup code will be inserted before the provided
    /// instruction which destroys all values that will be going out of scope. The instruction
    /// should typically be the final `call` instruction in the that is being exited.
    pub(crate) fn end_continuation_point_as_clean(&self, call_instruction: InstructionValue<'ctx>) {
        let mut cps = self.continuation_points.borrow_mut();
        let last = cps.last().unwrap();
        let mut next = last.chain();
        next.clean_from(
            last,
            call_instruction,
            self.builder.get_current_debug_location().unwrap(),
        );
        cps.push(Rc::new(next));
    }

    pub(crate) fn close_from(
        &self,
        brancher: &Brancher<'ctx>,
        instruction: InstructionValue<'ctx>,
        debug_location: DILocation<'ctx>,
    ) {
        let mut cps = self.continuation_points.borrow_mut();
        let mut next = brancher.0.chain();
        next.close_from(&brancher.0, instruction, debug_location);
        cps.push(Rc::new(next));
    }

    pub(crate) fn continue_from(&self, brancher: &Brancher<'ctx>) {
        let mut cps = self.continuation_points.borrow_mut();
        cps.push(brancher.0.clone());
    }

    pub(crate) fn capture_from(
        &self,
        brancher: &Brancher<'ctx>,
        instruction: InstructionValue<'ctx>,
        debug_location: DILocation<'ctx>,
    ) {
        let mut cps = self.continuation_points.borrow_mut();
        let mut next = brancher.0.chain();
        next.capture_from(&brancher.0, instruction, debug_location);
        cps.push(Rc::new(next));
    }

    pub(crate) fn branch(&self) -> Brancher<'ctx> {
        let parent = self.continuation_points.borrow().last().unwrap().clone();
        Brancher(parent)
    }

    pub(crate) fn merger(&self) -> Merger<'ctx> {
        Merger(vec![])
    }

    pub(crate) fn merge_into(
        &self,
        merger: &mut Merger<'ctx>,
        instruction: InstructionValue<'ctx>,
        debug_location: DILocation<'ctx>,
    ) {
        let cps = self.continuation_points.borrow();
        merger.close_from(cps.last().unwrap(), instruction, debug_location);
    }

    pub(crate) fn merge_branch(&self, branch: Brancher<'ctx>, merger: Merger<'ctx>) {
        let mut cps = self.continuation_points.borrow_mut();
        let mut cp = branch.0.chain();
        cp.parents = merger.0;
        cps.push(Rc::new(cp))
    }

    fn clean_and_close_scope(&self, cp: &ContinuationPoint<'ctx>) {
        for (id, var) in cp.variables.borrow().iter() {
            match var {
                Variable::Owned(pointer) if matches!(id, Closed::Variable(..)) => {
                    if let Some(pointer) = cp.upvalues.borrow().get(id) {
                        let upvalue = self.trilogy_reference_assume(*pointer);
                        self.trilogy_reference_close(upvalue);
                        self.trilogy_value_destroy(*pointer);
                    } else {
                        let instruction = self.trilogy_value_destroy(*pointer);
                        cp.unclosed
                            .borrow_mut()
                            .entry(*pointer)
                            .or_default()
                            .push(instruction);
                    }
                }
                Variable::Closed { upvalue, .. } => {
                    self.trilogy_value_destroy(*upvalue);
                }
                _ => {}
            }
        }
        for param in self.get_function().get_param_iter() {
            let param_ptr = self
                .builder
                .build_alloca(self.value_type(), "param")
                .unwrap();
            self.builder.build_store(param_ptr, param).unwrap();
            self.trilogy_value_destroy(param_ptr);
        }
    }

    pub(crate) fn close_continuation(&self) {
        while let Some(point) = {
            let mut rcs = self.continuation_points.borrow_mut();
            rcs.pop()
        } {
            let Some(point) = Rc::into_inner(point) else {
                continue;
            };
            for parent in &point.parents {
                match parent {
                    Exit::Close(Parent {
                        parent,
                        instruction,
                        debug_location,
                    }) => {
                        self.builder
                            .position_at(instruction.get_parent().unwrap(), instruction);
                        self.builder.set_current_debug_location(*debug_location);
                        let parent = parent.upgrade().unwrap();
                        let closure = self.build_closure(&parent, &point);
                        self.clean_and_close_scope(&parent);
                        instruction.replace_all_uses_with(&closure.as_instruction_value().unwrap());
                        instruction.erase_from_basic_block();
                    }
                    Exit::Clean(Parent {
                        parent,
                        instruction,
                        debug_location,
                    }) => {
                        self.builder.position_before(instruction);
                        self.builder.set_current_debug_location(*debug_location);
                        let parent = parent.upgrade().unwrap();
                        self.clean_and_close_scope(&parent);
                    }
                    Exit::Capture(Parent {
                        parent,
                        instruction,
                        debug_location,
                    }) => {
                        self.builder
                            .position_at(instruction.get_parent().unwrap(), instruction);
                        self.builder.set_current_debug_location(*debug_location);
                        let closure = self.build_closure(&parent.upgrade().unwrap(), &point);
                        instruction.replace_all_uses_with(&closure.as_instruction_value().unwrap());
                        instruction.erase_from_basic_block();
                    }
                }
            }
        }
    }

    fn build_closure(
        &self,
        scope: &ContinuationPoint<'ctx>,
        child_scope: &ContinuationPoint<'ctx>,
    ) -> PointerValue<'ctx> {
        let closure_size = child_scope.closure.borrow().len();
        let closure = self.allocate_value("closure");
        let closure_array = self.trilogy_array_init_cap(closure, closure_size, "closure.payload");
        let mut upvalues = scope.upvalues.borrow_mut();
        for id in child_scope.closure.borrow().iter() {
            let upvalue_name = format!("{id}.up");
            let new_upvalue = if let Some(ptr) = upvalues.get(id) {
                let new_upvalue = self.allocate_value(&upvalue_name);
                self.trilogy_value_clone_into(new_upvalue, *ptr);
                new_upvalue
            } else {
                match self
                    .reference_from_scope(scope, id)
                    .expect("closure is messed up")
                {
                    Variable::Closed { upvalue, .. } => {
                        let new_upvalue = self.allocate_value(&upvalue_name);
                        self.trilogy_value_clone_into(new_upvalue, upvalue);
                        new_upvalue
                    }
                    Variable::Owned(variable) => {
                        let builder = self.context.create_builder();
                        let declaration = variable.as_instruction_value().unwrap();
                        builder.position_at(
                            declaration.get_parent().unwrap(),
                            // NOTE: some reason this `position_at` seems to position BEFORE, not after as it is described, so we
                            // must position after the next instruction.
                            //
                            // We also actually want it to be after the storing of the 0, so we go two instructions forwards...
                            &declaration
                                .get_next_instruction()
                                .unwrap()
                                .get_next_instruction()
                                .unwrap(),
                        );
                        let original_upvalue = builder
                            .build_alloca(self.value_type(), &upvalue_name)
                            .unwrap();
                        builder
                            .build_store(original_upvalue, self.value_type().const_zero())
                            .unwrap();
                        let upvalue_internal =
                            self.trilogy_reference_to_in(&builder, original_upvalue, variable);
                        upvalues.insert(id.clone(), original_upvalue);

                        if let Some(closing) = scope.unclosed.borrow_mut().remove(&variable) {
                            // Due to the order of the code, captures appear above closes and cleans for
                            // the same parent in the continuation_points list.
                            //
                            // Really, what we want to do is to build all the capturing closures before
                            // building the cleaning/closing closures, so that those ones have the upvalues
                            // list set properly... but since that's not that easy, we just store the list
                            // of unclosed destroyed variables and close them if necessary
                            for instruction in closing {
                                builder.position_before(&instruction);
                                self.trilogy_reference_close_in(&builder, upvalue_internal);
                            }
                        }

                        let new_upvalue = self.allocate_value(&upvalue_name);
                        self.trilogy_value_clone_into(new_upvalue, original_upvalue);
                        new_upvalue
                    }
                }
            };
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

    fn current_continuation(&self) -> Rc<ContinuationPoint<'ctx>> {
        self.continuation_points.borrow().last().unwrap().clone()
    }

    pub(crate) fn get_variable(&self, id: &Id) -> Option<Variable<'ctx>> {
        self.get_variable_from(&self.current_continuation(), id)
    }

    fn get_variable_from(
        &self,
        scope: &ContinuationPoint<'ctx>,
        id: &Id,
    ) -> Option<Variable<'ctx>> {
        self.reference_from_scope(scope, &Closed::Variable(id.clone()))
    }

    pub(crate) fn use_temporary(&self, temporary: PointerValue<'ctx>) -> Option<Variable<'ctx>> {
        let cp = self.current_continuation();
        self.reference_from_scope(&cp, &Closed::Temporary(temporary))
    }

    fn reference_from_scope(
        &self,
        scope: &ContinuationPoint<'ctx>,
        closed: &Closed<'ctx>,
    ) -> Option<Variable<'ctx>> {
        if let Some(var) = scope.variables.borrow().get(closed) {
            return Some(*var);
        }

        if scope.parent_variables.contains(closed) {
            let mut closure = scope.closure.borrow_mut();
            let closure_index = closure.len();
            closure.push(closed.clone());

            let builder = self.context.create_builder();
            let entry = self.get_function().get_first_basic_block().unwrap();
            match entry.get_first_instruction() {
                Some(instruction) => builder.position_before(&instruction),
                None => builder.position_at_end(entry),
            }
            builder.set_current_debug_location(self.builder.get_current_debug_location().unwrap());

            let upvalue =
                self.get_closure_upvalue(&builder, closure_index, &format!("{closed}.up"));
            let ref_value = self.trilogy_reference_assume_in(&builder, upvalue);
            let ptr_to_location = builder
                .build_struct_gep(self.reference_value_type(), ref_value, 1, "")
                .unwrap();
            let location = builder
                .build_load(
                    self.context.ptr_type(AddressSpace::default()),
                    ptr_to_location,
                    &closed.to_string(),
                )
                .unwrap()
                .into_pointer_value();
            let variable = Variable::Closed { location, upvalue };
            scope
                .variables
                .borrow_mut()
                .insert(closed.clone(), variable);
            return Some(variable);
        }

        None
    }

    pub(crate) fn bind_temporary(&self, temporary: PointerValue<'ctx>) {
        let cp = self.current_continuation();
        cp.variables
            .borrow_mut()
            .insert(Closed::Temporary(temporary), Variable::Owned(temporary));
    }

    pub(crate) fn variable(&self, id: &ir::Identifier) -> PointerValue<'ctx> {
        if let Some(variable) = self.get_variable(&id.id) {
            return variable.ptr();
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
            .insert(Closed::Variable(id.id.clone()), Variable::Owned(variable));

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
