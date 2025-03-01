use crate::debug_info::DebugInfo;
use crate::types;
use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;
use inkwell::values::{BasicValue, PointerValue};
use source_span::Span;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use trilogy_ir::{Id, ir};

mod continuation_point;
mod variables;

pub(crate) use continuation_point::Merger;
use continuation_point::{ContinuationPoint, Exit, Parent};
use variables::Closed;
pub(crate) use variables::{Head, Variable};

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
