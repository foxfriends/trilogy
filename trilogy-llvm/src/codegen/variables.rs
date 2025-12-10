//! Functions for accessing variables and values. The availability of variables is highly
//! dependent on the continuation point and its parents.
use super::{Codegen, ContinuationPoint};
use inkwell::AddressSpace;
use inkwell::builder::Builder;
use inkwell::values::{FunctionValue, PointerValue};
use trilogy_ir::Id;

/// Represents a referenced variable, which may either be owned by the current scope, or
/// previously closed over and now being read from the closure.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Variable<'ctx> {
    /// This is a variable that was closed.
    Closed {
        /// The pointer to the actual underlying value. This is the same as the location field
        /// of the upvalue.
        location: PointerValue<'ctx>,
        /// The pointer to the upvalue for this closed value.
        upvalue: PointerValue<'ctx>,
    },
    /// This is a variable that was defined in the current continuation point and is still owned.
    Owned(PointerValue<'ctx>),
    /// This is a variable that was defined in the current continuation point and is still owned.
    Argument(PointerValue<'ctx>),
}

impl<'ctx> Variable<'ctx> {
    pub(crate) fn ptr(&self) -> PointerValue<'ctx> {
        match self {
            Self::Closed { location, .. } => *location,
            Self::Owned(ptr) => *ptr,
            Self::Argument(ptr) => *ptr,
        }
    }
}

/// Represents any value that can be closed over by a closure.
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub(super) enum Closed<'ctx> {
    /// This is a named variable from the source code that is captured by the closure.
    Variable(Id),
    /// This is an intermediate temporary value that was explicitly bound for capturing.
    Temporary(PointerValue<'ctx>),
}

#[derive(Clone, Debug)]
pub(crate) enum Head {
    Constant,
    Function,
    Procedure,
    Rule,
    Module,
    ExternalModule(String),
}

#[derive(Clone, Debug)]
pub(crate) struct Global {
    pub path: Vec<String>,
    pub id: Id,
    pub head: Head,
}

impl Global {
    pub(crate) fn module_path(&self, relative_to: &str) -> String {
        self.path
            .iter()
            .fold(relative_to.to_owned(), |f, p| format!("{f}::{p}"))
    }
}

impl std::fmt::Display for Closed<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Variable(id) => id.fmt(f),
            Self::Temporary(ptr) => write!(f, "{}.ref", ptr.get_name().to_str().unwrap()),
        }
    }
}

impl<'ctx> Codegen<'ctx> {
    /// Gets the current LLVM function that we are in.
    pub(crate) fn get_function(&self) -> FunctionValue<'ctx> {
        self.builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap()
    }

    /// Gets the `return_to` pointer from the current function context.
    pub(crate) fn get_return(&self, name: &str) -> PointerValue<'ctx> {
        let container = self.allocate_value(name);
        self.clone_return(container);
        container
    }

    pub(crate) fn get_return_temporary(&self) -> PointerValue<'ctx> {
        self.function_params.borrow()[0]
    }

    /// Gets the `return_to` pointer from the current function context.
    pub(crate) fn clone_return(&self, into: PointerValue<'ctx>) {
        self.trilogy_value_clone_into(into, self.get_return_temporary());
    }

    /// Gets the `yield_to` pointer from the current function context.
    pub(crate) fn get_yield(&self, name: &str) -> PointerValue<'ctx> {
        let container = self.allocate_value(name);
        self.clone_yield(container);
        container
    }

    /// Gets the `yield_to` pointer from the current function context.
    pub(crate) fn clone_yield(&self, into: PointerValue<'ctx>) {
        self.trilogy_value_clone_into(into, self.function_params.borrow()[1]);
    }

    /// Gets the `end_to` pointer from the current function context.
    pub(crate) fn get_end(&self, name: &str) -> PointerValue<'ctx> {
        let container = self.allocate_value(name);
        self.clone_end(container);
        container
    }

    pub(crate) fn get_end_temporary(&self) -> PointerValue<'ctx> {
        self.function_params.borrow()[1]
    }

    /// Gets the `end_to` pointer from the current function context.
    pub(crate) fn clone_end(&self, into: PointerValue<'ctx>) {
        self.trilogy_value_clone_into(into, self.function_params.borrow()[2]);
    }

    /// Gets the `next_to` pointer from the current rule context.
    pub(crate) fn get_next(&self, name: &str) -> PointerValue<'ctx> {
        let container = self.allocate_value(name);
        self.clone_next(container);
        container
    }

    /// Gets the current `next` continuation, valid only when in a query.
    pub(crate) fn clone_next(&self, into: PointerValue<'ctx>) {
        self.trilogy_value_clone_into(into, self.function_params.borrow()[3]);
    }

    /// Gets the `done_to` pointer from the current rule context.
    pub(crate) fn get_done(&self, name: &str) -> PointerValue<'ctx> {
        let container = self.allocate_value(name);
        self.clone_done(container);
        container
    }

    /// Gets the current `done` continuation, valid only when in a query.
    pub(crate) fn clone_done(&self, into: PointerValue<'ctx>) {
        self.trilogy_value_clone_into(into, self.function_params.borrow()[4]);
    }

    /// When in a continuation function, gets the value that was yielded to the continuation.
    pub(crate) fn get_continuation(&self, name: &str) -> PointerValue<'ctx> {
        let container = self.allocate_value(name);
        self.trilogy_value_clone_into(container, self.function_params.borrow()[5]);
        container
    }

    /// When in an effect handler, get the yielded effect.
    pub(crate) fn get_effect_temporary(&self) -> PointerValue<'ctx> {
        self.function_params.borrow()[5]
    }

    /// When in an effect handler, get the resume_to continuation. This is a direct pointer,
    /// not a clone of the value.
    pub(crate) fn get_provided_resume(&self) -> PointerValue<'ctx> {
        self.function_params.borrow()[6]
    }

    /// When in a handler function, gets the cancel to pointer.
    pub(crate) fn get_cancel(&self) -> PointerValue<'ctx> {
        let current_cancel = *self.current_cancel.borrow().last().unwrap();
        self.use_temporary(current_cancel).unwrap()
    }

    /// When in a handler function, gets the resume to pointer.
    pub(crate) fn get_resume(&self) -> PointerValue<'ctx> {
        let current_resume = *self.current_resume.borrow().last().unwrap();
        self.use_temporary(current_resume).unwrap()
    }

    /// Gets the current `break` continuation, valid only when in a loop.
    pub(crate) fn get_break(&self) -> PointerValue<'ctx> {
        let current_break = *self.current_break.borrow().last().unwrap();
        self.use_temporary_clone(current_break).unwrap()
    }

    /// Gets the current `continue` continuation, valid only when in a loop.
    pub(crate) fn get_continue(&self) -> PointerValue<'ctx> {
        let current_continue = *self.current_continue.borrow().last().unwrap();
        self.use_temporary_clone(current_continue).unwrap()
    }

    pub(crate) fn get_closure(&self, name: &str) -> PointerValue<'ctx> {
        let container = self.allocate_value(name);
        self.trilogy_value_clone_into(container, *self.function_params.borrow().last().unwrap());
        container
    }

    fn get_closure_array(&self, builder: &Builder<'ctx>) -> PointerValue<'ctx> {
        match self.closure_array.get() {
            Some(array) => array,
            None => {
                let closure_ptr = self.function_params.borrow().last().copied().unwrap();
                match closure_ptr.as_instruction() {
                    Some(instruction) => {
                        // <- this is where instruction points
                        // %closure_ptr = alloca
                        // store %closure, %closure_ptr
                        // <- this is where we want to be
                        if let Some(instruction) = instruction
                            .get_next_instruction()
                            .and_then(|ins| ins.get_next_instruction())
                        {
                            builder.position_at(instruction.get_parent().unwrap(), &instruction);
                        } else {
                            builder.position_at_end(instruction.get_parent().unwrap());
                        }
                    }
                    None => {
                        let block = self.get_function().get_first_basic_block().unwrap();
                        match block.get_first_instruction() {
                            Some(instruction) => {
                                builder.position_at(block, &instruction);
                            }
                            None => {
                                builder.position_at_end(block);
                            }
                        }
                    }
                };
                let closure_array =
                    self.trilogy_array_assume_in(builder, closure_ptr, "closure_array");
                self.closure_array.set(Some(closure_array));
                closure_array
            }
        }
    }

    /// When in a closure, retrieves an upvalue from the captured closure.
    ///
    /// The closure is always passed as the last parameter, and is a Trilogy array of Trilogy references.
    /// To access a variable from the closure:
    /// 1. Consider the nth element of the array
    /// 2. Get the value inside
    /// 3. Assume a reference, and load its location field
    /// 4. That value of that location field is the pointer to the actual value
    fn get_closure_upvalue(
        &self,
        builder: &Builder<'ctx>,
        index: usize,
        name: &str,
    ) -> PointerValue<'ctx> {
        let closure_array = self.get_closure_array(builder);
        let instruction = closure_array.as_instruction().unwrap();
        if let Some(instruction) = instruction.get_next_instruction() {
            builder.position_at(instruction.get_parent().unwrap(), &instruction);
        }
        let upvalue = builder.build_alloca(self.value_type(), name).unwrap();
        builder
            .build_store(upvalue, self.value_type().const_zero())
            .unwrap();
        self.trilogy_array_at_in(builder, upvalue, closure_array, index);
        upvalue
    }

    /// Records a temporary value in the current continuation, allowing it to be later
    /// used. If a temporary is captured and used in this way, it will be added to the
    /// closure, if necessary.
    pub(crate) fn bind_temporary(&self, temporary: PointerValue<'ctx>) {
        let cp = self.current_continuation_point();
        if !cp.contains_temporary(temporary) {
            cp.variables
                .borrow_mut()
                .insert(Closed::Temporary(temporary), Variable::Owned(temporary));
        }
    }

    /// Records an argument value as a temporary in the current continuation. Similar to
    /// binding a regular temporary, but argument values are cleaned up slightly differently.
    pub(crate) fn bind_argument(&self, temporary: PointerValue<'ctx>) {
        let cp = self.current_continuation_point();
        if !cp.contains_temporary(temporary) {
            cp.variables
                .borrow_mut()
                .insert(Closed::Temporary(temporary), Variable::Argument(temporary));
        }
    }

    /// Uses a previously bound temporary value. If the value was not previously bound with
    /// `bind_temporary`, this will return `None`.
    pub(crate) fn use_temporary(
        &self,
        temporary: PointerValue<'ctx>,
    ) -> Option<PointerValue<'ctx>> {
        match self.reference_from_scope(
            &self.current_continuation_point(),
            &Closed::Temporary(temporary),
        )? {
            Variable::Owned(pointer) => Some(pointer),
            Variable::Argument(pointer) => Some(pointer),
            Variable::Closed { location, .. } => Some(location),
        }
    }

    /// Uses a clone of a previously bound temporary value.
    pub(crate) fn use_temporary_clone(
        &self,
        temporary: PointerValue<'ctx>,
    ) -> Option<PointerValue<'ctx>> {
        match self.reference_from_scope(
            &self.current_continuation_point(),
            &Closed::Temporary(temporary),
        )? {
            Variable::Owned(pointer) => Some(pointer),
            Variable::Argument(pointer) => Some(pointer),
            Variable::Closed { location, .. } => {
                let clone = self.allocate_value("");
                self.trilogy_value_clone_into(clone, location);
                Some(clone)
            }
        }
    }

    /// Uses a previously bound temporary value, only if it is not captured.
    fn use_owned_temporary(&self, temporary: PointerValue<'ctx>) -> Option<PointerValue<'ctx>> {
        match self.reference_from_scope(
            &self.current_continuation_point(),
            &Closed::Temporary(temporary),
        )? {
            Variable::Owned(pointer) => Some(pointer),
            Variable::Argument(pointer) => Some(pointer),
            Variable::Closed { .. } => None,
        }
    }

    /// Destroy a previously bound temporary value, if it was not captured by this point.
    /// Often temporaries need to be destroyed but only if they are not in a closure,
    /// as the closure will handle destruction of its internal values.
    pub(crate) fn destroy_owned_temporary(&self, temporary: PointerValue<'ctx>) {
        if let Some(temp) = self.use_owned_temporary(temporary) {
            self.trilogy_value_destroy(temp);
        }
    }

    /// Gets a variable from the current scope. Any variable that is defined in the current continuation point
    /// or is visible from the parent continuation point can be referenced in this way.
    pub(crate) fn get_variable(&self, id: &Id) -> Option<Variable<'ctx>> {
        self.reference_from_scope(
            &self.current_continuation_point(),
            &Closed::Variable(id.clone()),
        )
    }

    /// Gets a variable or temporary from a particular scope.
    pub(super) fn reference_from_scope(
        &self,
        scope: &ContinuationPoint<'ctx>,
        closed: &Closed<'ctx>,
    ) -> Option<Variable<'ctx>> {
        // If the variable has already been referenced in the current scope, return the saved reference to avoid
        // doing the work of looking it up again.
        //
        // This is the case for all owned variables, and also for closed variables that have already been used.
        if let Some(var) = scope.variables.borrow().get(closed) {
            return Some(*var);
        }

        if let Some(shadowed) = &scope.shadows {
            // If the current scope is shadowing another, then we defer to the shadowed one at this point, as
            // the parent variables must always be collected in the top-most scope (the REAL scope).
            return self.reference_from_scope(&shadowed.upgrade().unwrap(), closed);
        }

        if scope.parent_variables.contains(closed) {
            // Otherwise, the variable might be visible from the parent continuation point.
            //
            // In this case, we first update the closure to ensure that we know to close over this variable
            // before exiting the parent scope.
            let mut closure = scope.closure.borrow_mut();
            let closure_index = if let Some(index) = closure.iter().position(|v| v == closed) {
                index
            } else {
                let index = closure.len();
                closure.push(closed.clone());
                index
            };

            // As recommended by the LLVM docs somewhere, we prefer to hoist variable declarations to
            // the first basic block of the function.
            let builder = self.context.create_builder();
            let entry = self.get_function().get_first_basic_block().unwrap();
            match entry.get_first_instruction() {
                Some(instruction) => builder.position_before(&instruction),
                None => builder.position_at_end(entry),
            }
            builder.set_current_debug_location(self.builder.get_current_debug_location().unwrap());

            // Since we declared the intended position of this variable in the closure, that's where we'll
            // pull it from.
            let upvalue =
                self.get_closure_upvalue(&builder, closure_index, &format!("{closed}.up"));
            let reference = self.trilogy_reference_assume_in(&builder, upvalue);
            let location =
                self.trilogy_reference_get_location_in(&builder, reference, &closed.to_string());
            // Then we record that we have already located this variable, to avoid relocating it if referenced
            // again from the current continuation point.
            let variable = Variable::Closed { location, upvalue };
            if let Closed::Variable(var) = closed {
                // Add debug information to this variable as if it was a declaration,
                // same as we do for newly declared variables below.
                self.di.describe_variable(
                    location,
                    var.name(),
                    var.declaration_span,
                    &builder,
                    self.get_function().get_subprogram().unwrap(),
                    self.create_debug_location(var.declaration_span),
                );
            }
            scope
                .variables
                .borrow_mut()
                .insert(closed.clone(), variable);
            return Some(variable);
        }

        None
    }

    fn trilogy_reference_get_location_in(
        &self,
        builder: &Builder<'ctx>,
        reference: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let ptr_to_location = builder
            // Field 1 is location, according to types.h
            .build_struct_gep(self.reference_value_type(), reference, 1, "")
            .unwrap();
        builder
            .build_load(
                self.context.ptr_type(AddressSpace::default()),
                ptr_to_location,
                name,
            )
            .unwrap()
            .into_pointer_value()
    }

    pub(crate) fn trilogy_reference_get_location(
        &self,
        upvalue: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        self.trilogy_reference_get_location_in(&self.builder, upvalue, name)
    }

    /// References a variable, if it is already available, or defines a it in the current scope otherwise.
    pub(crate) fn variable(&self, id: &Id) -> PointerValue<'ctx> {
        // If the variable is already available, just return the existing reference.
        if let Some(variable) = self.get_variable(id) {
            return variable.ptr();
        }

        // As recommended by the LLVM docs somewhere, we prefer to hoist variable declarations to
        // the first basic block of the function.
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
        // Add this variable as an owned variable in the current continuation point.
        self.current_continuation_point()
            .shadow_root()
            .variables
            .borrow_mut()
            .insert(Closed::Variable(id.clone()), Variable::Owned(variable));

        self.di.describe_variable(
            variable,
            id.name(),
            id.declaration_span,
            &builder,
            self.get_function().get_subprogram().unwrap(),
            self.create_debug_location(id.declaration_span),
        );

        variable
    }
}
