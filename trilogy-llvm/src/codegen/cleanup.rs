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
use inkwell::debug_info::DILocation;
use inkwell::intrinsics::Intrinsic;
use inkwell::values::{BasicValue, InstructionValue, PointerValue};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Default)]
struct CleanupState<'ctx> {
    /// The mapping from variable names to their upvalues. If one already exists for a variable
    /// as it is being captured, it must be reused.
    upvalues: HashMap<Closed<'ctx>, PointerValue<'ctx>>,

    /// This is tracking all places that a variable is destroyed during scope cleanup without
    /// being closed. If we later determine that we need to close that variable, this allows
    /// us to go back and make sure it was closed after all.
    unclosed: HashMap<PointerValue<'ctx>, Vec<(InstructionValue<'ctx>, DILocation<'ctx>)>>,

    /// Similarly, this tracks all places where we do cleanup, and the upvalues which were
    /// handled at those points. Since the upvalues in some scope may be discovered after
    /// that scope has already been cleaned, we need to revisit those scopes to close such
    /// upvalues.
    handled: Vec<(
        InstructionValue<'ctx>,
        DILocation<'ctx>,
        HashSet<PointerValue<'ctx>>,
    )>,
}

impl<'ctx> Codegen<'ctx> {
    fn do_nothing(&self) -> InstructionValue<'ctx> {
        let do_nothing = Intrinsic::find("llvm.donothing").unwrap();
        let do_nothing = do_nothing.get_declaration(&self.module, &[]).unwrap();
        self.builder
            .build_call(do_nothing, &[], "noop")
            .unwrap()
            .try_as_basic_value()
            .unwrap_instruction()
    }

    /// Closes the current continuation point and all the continuation points up the chain.
    /// This is intended to be called at the end of a top level declaration.
    pub(crate) fn close_continuation(&self) {
        let mut cleanup_state = HashMap::<*const ContinuationPoint<'ctx>, CleanupState>::new();

        while let Some(point) = {
            let mut rcs = self.continuation_points.borrow_mut();
            rcs.pop()
        } {
            let point_pointer = Rc::as_ptr(&point);
            let Some(point) = Rc::into_inner(point) else {
                continue;
            };
            for parent in &point.parents {
                match parent {
                    Exit::Close(Parent {
                        parent,
                        closure_instruction,
                        close_after_instruction,
                        snapshot,
                    }) => {
                        // Allocate the closure in the same spot as the original allocation.
                        self.restore_function_context(snapshot.clone());
                        self.builder.position_at(
                            closure_instruction.get_parent().unwrap(),
                            closure_instruction,
                        );
                        let closure_size = point.closure.borrow().len();
                        let closure = self.allocate_value("closure");
                        let parent = parent.upgrade().unwrap();
                        let mut handled_upvalues = HashSet::new();
                        if closure_size > 0 {
                            let closure_array = self.trilogy_array_init_cap(
                                closure,
                                closure_size,
                                "closure.payload",
                            );
                            // But close it after the close_after_instruction
                            self.builder.position_at(
                                close_after_instruction.get_parent().unwrap(),
                                &close_after_instruction.get_next_instruction().unwrap(),
                            );
                            self.build_closure(
                                &mut cleanup_state,
                                &mut handled_upvalues,
                                closure_array,
                                parent.clone(),
                                &point,
                            );
                        }
                        self.clean_and_close_scope(
                            &mut cleanup_state,
                            &mut handled_upvalues,
                            &parent,
                        );

                        cleanup_state
                            .entry(Rc::as_ptr(&parent.shadow_root()))
                            .or_default()
                            .handled
                            .push((
                                self.do_nothing(),
                                self.builder.get_current_debug_location().unwrap(),
                                handled_upvalues,
                            ));

                        closure_instruction
                            .replace_all_uses_with(&closure.as_instruction_value().unwrap());
                        closure_instruction.erase_from_basic_block();
                    }
                    Exit::Clean(Parent {
                        parent,
                        closure_instruction,
                        close_after_instruction: _,
                        snapshot,
                    }) => {
                        self.builder.position_before(closure_instruction);
                        self.restore_function_context(snapshot.clone());
                        let parent = parent.upgrade().unwrap();
                        let mut handled_upvalues = HashSet::new();
                        self.clean_and_close_scope(
                            &mut cleanup_state,
                            &mut handled_upvalues,
                            &parent,
                        );

                        cleanup_state
                            .entry(Rc::as_ptr(&parent.shadow_root()))
                            .or_default()
                            .handled
                            .push((
                                self.do_nothing(),
                                self.builder.get_current_debug_location().unwrap(),
                                handled_upvalues,
                            ));
                    }
                    Exit::Capture(Parent {
                        parent,
                        closure_instruction,
                        close_after_instruction,
                        snapshot,
                    }) => {
                        // Allocate the closure in the same spot as the original allocation.
                        self.restore_function_context(snapshot.clone());
                        self.builder.position_at(
                            closure_instruction.get_parent().unwrap(),
                            closure_instruction,
                        );
                        let closure_size = point.closure.borrow().len();
                        let closure = self.allocate_value("closure");
                        if closure_size > 0 {
                            let closure_array = self.trilogy_array_init_cap(
                                closure,
                                closure_size,
                                "closure.payload",
                            );
                            // But close it after the close_after_instruction
                            self.builder.position_at(
                                close_after_instruction.get_parent().unwrap(),
                                &close_after_instruction.get_next_instruction().unwrap(),
                            );
                            self.restore_function_context(snapshot.clone());
                            let parent = parent.upgrade().unwrap();
                            self.build_closure(
                                &mut cleanup_state,
                                &mut HashSet::new(),
                                closure_array,
                                parent,
                                &point,
                            );
                        }
                        closure_instruction
                            .replace_all_uses_with(&closure.as_instruction_value().unwrap());
                        closure_instruction.erase_from_basic_block();
                    }
                }
            }
            // If this point's upvalue array is populated, that means it's a shadow root, and this
            // is the last time we will visit this shadow root. We now need to revisit all of the children
            // of point and close any upvalues that were created after we closed that child.
            let Some(state) = cleanup_state.get(&point_pointer) else {
                continue;
            };
            if state.upvalues.is_empty() {
                continue;
            }
            for (location, di_location, handled) in &state.handled {
                self.builder
                    .position_at(location.get_parent().unwrap(), location);
                self.builder.set_current_debug_location(*di_location);
                for upvalue in state.upvalues.values() {
                    if !handled.contains(upvalue) {
                        self.trilogy_value_destroy(*upvalue);
                    }
                }
                location.remove_from_basic_block();
            }
        }
    }

    fn clean_and_close_scope(
        &self,
        state: &mut HashMap<*const ContinuationPoint<'ctx>, CleanupState<'ctx>>,
        handled_upvalues: &mut HashSet<PointerValue<'ctx>>,
        cp: &Rc<ContinuationPoint<'ctx>>,
    ) {
        if let Some(shadowed) = &cp.shadows {
            self.clean_and_close_scope(state, handled_upvalues, &shadowed.upgrade().unwrap());
        }
        let root_state = state.entry(Rc::as_ptr(&cp.shadow_root())).or_default();

        for (id, var) in cp.variables.borrow().iter() {
            match var {
                Variable::Owned(pointer) => {
                    if let Some(pointer) = root_state.upvalues.get(id) {
                        // We have detected this variable as referenced in a future scope, so we have to close it
                        let upvalue = self.trilogy_reference_assume(*pointer);
                        self.trilogy_reference_close(upvalue);
                        self.trilogy_value_destroy(*pointer);
                        handled_upvalues.insert(*pointer);
                    } else if matches!(id, Closed::Variable(..)) {
                        // In this case, we have not YET detected that it is referenced, but it still might be
                        // detected later, so we have to record this destroy in case it has to be upgraded to a
                        // "close".
                        let instruction = self.trilogy_value_destroy(*pointer);
                        root_state.unclosed.entry(*pointer).or_default().push((
                            instruction,
                            self.builder.get_current_debug_location().unwrap(),
                        ));
                    } else {
                        // Similarly, but for temporaries: we don't need to explicitly destroy them because
                        // their destruction (or lack thereof) is expected by the rest of codegen. We do,
                        // however, wish to track them for closing purposes, so use a no-op instead of a destroy.
                        // self.allocate_value(&format!("comment#{id:?}"));
                        let instruction = self.do_nothing();
                        root_state.unclosed.entry(*pointer).or_default().push((
                            instruction,
                            self.builder.get_current_debug_location().unwrap(),
                        ));
                    }
                }
                // Function arguments are much the same as variables, but are never temporaries despite being
                // closed as anonymous pointers and therefore looking like temporaries on the `id` side.
                Variable::Argument(pointer) => {
                    if let Some(pointer) = root_state.upvalues.get(id) {
                        // We have detected this variable as referenced in a future scope, so we have to close it
                        let upvalue = self.trilogy_reference_assume(*pointer);
                        self.trilogy_reference_close(upvalue);
                        self.trilogy_value_destroy(*pointer);
                        handled_upvalues.insert(*pointer);
                    } else {
                        // In this case, we have not YET detected that it is referenced, but it still might be
                        // detected later, so we have to record this destroy in case it has to be upgraded to a
                        // "close".
                        let instruction = self.trilogy_value_destroy(*pointer);
                        root_state.unclosed.entry(*pointer).or_default().push((
                            instruction,
                            self.builder.get_current_debug_location().unwrap(),
                        ));
                    }
                }
                Variable::Closed { upvalue, .. } => {
                    // Variable was closed in a further parent scope, so does not need to be re-closed
                    self.trilogy_value_destroy(*upvalue);
                }
            }
        }
    }

    fn build_closure(
        &self,
        state: &mut HashMap<*const ContinuationPoint<'ctx>, CleanupState<'ctx>>,
        handled_upvalues: &mut HashSet<PointerValue<'ctx>>,
        closure_array: PointerValue<'ctx>,
        scope: Rc<ContinuationPoint<'ctx>>,
        child_scope: &ContinuationPoint<'ctx>,
    ) {
        let root_scope = scope.shadow_root();
        let root_state = state.entry(Rc::as_ptr(&root_scope)).or_default();

        for id in child_scope.closure.borrow().iter() {
            let new_upvalue = if let Some(ptr) = root_state.upvalues.get(id) {
                let new_upvalue = self.allocate_value(&format!("{id}.cloneup"));
                self.trilogy_value_clone_into(new_upvalue, *ptr);
                new_upvalue
            } else {
                match self
                    .reference_from_scope(scope.as_ref(), id)
                    .expect("closure is messed up")
                {
                    Variable::Closed { upvalue, .. } => {
                        let new_upvalue = self.allocate_value(&format!("{id}.reup"));
                        self.trilogy_value_clone_into(new_upvalue, upvalue);
                        new_upvalue
                    }
                    Variable::Owned(variable) | Variable::Argument(variable) => {
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
                            .build_alloca(self.value_type(), &format!("{id}.firstup"))
                            .unwrap();
                        builder
                            .build_store(original_upvalue, self.value_type().const_zero())
                            .unwrap();
                        let upvalue_internal = self.trilogy_reference_to_in(
                            &builder,
                            original_upvalue,
                            variable,
                            &format!("*{id}.firstup"),
                        );
                        root_state.upvalues.insert(id.clone(), original_upvalue);

                        assert_eq!(
                            builder.get_insert_block().unwrap().get_parent(),
                            self.builder.get_insert_block().unwrap().get_parent()
                        );

                        if let Some(closing) = root_state.unclosed.remove(&variable) {
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
                                self.trilogy_value_destroy_in(&builder, original_upvalue);
                                handled_upvalues.insert(original_upvalue);
                            }
                        }

                        let new_upvalue = self.allocate_value(&format!("{id}.newup"));
                        self.trilogy_value_clone_into(new_upvalue, original_upvalue);
                        new_upvalue
                    }
                }
            };
            self.trilogy_array_push(closure_array, new_upvalue);
        }
    }
}
