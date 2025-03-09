//! Implements the "clean up and close" phase of declaration compilation.
//!
//! During cleanup phase, we walk up the continuation point list in reverse, closing those
//! continuations. We do this in reverse to be able to propagate references from continuation
//! points later in the chain to their parents, as those parents must also reference those
//! values, until we reach the scope in which the value is defined.
//!
//! The one notable exception is for do/fn closures, as such closures are compiled first
//! without being able to tell continuation points created later which variables are referenced.
//! To facilitate this, while closing continuations we also record any variables which were
//! discarded without being captured, and retroactively close a created upvalue over those
//! variables as needed.
use super::{Closed, Codegen, ContinuationPoint, Exit, Parent, Variable};
use inkwell::values::{BasicValue, PointerValue};
use std::rc::Rc;

impl<'ctx> Codegen<'ctx> {
    /// Closes the current continuation point and all the continuation points up the chain.
    /// This is intended to be called at the end of a top level declaration.
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
                        snapshot,
                    }) => {
                        self.builder
                            .position_at(instruction.get_parent().unwrap(), instruction);
                        self.restore_function_context(snapshot.clone());
                        let parent = parent.upgrade().unwrap();
                        let closure = self.build_closure(&parent, &point);
                        self.clean_and_close_scope(&parent);
                        instruction.replace_all_uses_with(&closure.as_instruction_value().unwrap());
                        instruction.erase_from_basic_block();
                    }
                    Exit::Clean(Parent {
                        parent,
                        instruction,
                        snapshot,
                    }) => {
                        self.builder.position_before(instruction);
                        self.restore_function_context(snapshot.clone());
                        let parent = parent.upgrade().unwrap();
                        self.clean_and_close_scope(&parent);
                    }
                    Exit::Capture(Parent {
                        parent,
                        instruction,
                        snapshot,
                    }) => {
                        self.builder
                            .position_at(instruction.get_parent().unwrap(), instruction);
                        self.restore_function_context(snapshot.clone());
                        let closure = self.build_closure(&parent.upgrade().unwrap(), &point);
                        instruction.replace_all_uses_with(&closure.as_instruction_value().unwrap());
                        instruction.erase_from_basic_block();
                    }
                }
            }
        }
    }

    fn clean_and_close_scope(&self, cp: &ContinuationPoint<'ctx>) {
        for (id, var) in cp.variables.borrow().iter() {
            match var {
                Variable::Owned(pointer) if matches!(id, Closed::Variable(..)) => {
                    if let Some(pointer) = cp.upvalues.borrow().get(id) {
                        let upvalue = self.trilogy_reference_assume(*pointer);
                        self.trilogy_reference_close(upvalue);
                    } else {
                        let instruction = self.trilogy_value_destroy(*pointer);
                        cp.unclosed.borrow_mut().entry(*pointer).or_default().push((
                            instruction,
                            self.builder.get_current_debug_location().unwrap(),
                        ));
                    }
                }
                Variable::Closed { upvalue, .. } => {
                    self.trilogy_value_destroy(*upvalue);
                }
                _ => {}
            }
        }
        for param in self.function_params.borrow().iter() {
            self.trilogy_value_destroy(*param);
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
                            let debug_location = self.builder.get_current_debug_location().unwrap();
                            // Due to the order of the code, captures appear above closes and cleans for
                            // the same parent in the continuation_points list.
                            //
                            // Really, what we want to do is to build all the capturing closures before
                            // building the cleaning/closing closures, so that those ones have the upvalues
                            // list set properly... but since that's not that easy, we just store the list
                            // of unclosed destroyed variables and close them if necessary
                            for (instruction, di_location) in closing {
                                builder.position_before(&instruction);
                                builder.set_current_debug_location(di_location);
                                self.trilogy_reference_close_in(&builder, upvalue_internal);
                                instruction.remove_from_basic_block();
                            }
                            self.builder.set_current_debug_location(debug_location);
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
}
