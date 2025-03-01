use super::{Codegen, ContinuationPoint};
use inkwell::AddressSpace;
use inkwell::builder::Builder;
use inkwell::debug_info::AsDIScope;
use inkwell::llvm_sys::debuginfo::LLVMDIFlagPublic;
use inkwell::values::{FunctionValue, PointerValue};
use trilogy_ir::{Id, ir};

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
pub(super) enum Closed<'ctx> {
    Variable(Id),
    Temporary(PointerValue<'ctx>),
}

#[derive(Clone)]
#[expect(dead_code)]
pub(crate) enum Head {
    Constant,
    Function,
    Procedure,
    Rule,
    Module(String),
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

    pub(crate) fn bind_temporary(&self, temporary: PointerValue<'ctx>) {
        let cp = self.current_continuation();
        cp.variables
            .borrow_mut()
            .insert(Closed::Temporary(temporary), Variable::Owned(temporary));
    }

    pub(crate) fn use_temporary(&self, temporary: PointerValue<'ctx>) -> Option<Variable<'ctx>> {
        let cp = self.current_continuation();
        self.reference_from_scope(&cp, &Closed::Temporary(temporary))
    }

    pub(super) fn reference_from_scope(
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
}
