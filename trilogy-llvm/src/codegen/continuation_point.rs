//! Functions for managing context and scope across continuation points.

use super::{Closed, Codegen, Snapshot, Variable};
use inkwell::values::{InstructionValue, PointerValue};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicUsize, Ordering};
use trilogy_ir::Id;

#[derive(Clone, Debug)]
pub(super) struct Parent<'ctx> {
    /// A pointer to the continuation point that we are cleaning or closing from.
    pub parent: Weak<ContinuationPoint<'ctx>>,
    /// The instruction that corresponds to the closure being closed, if any.
    /// If none, just put whatever instruction.
    pub closure_instruction: InstructionValue<'ctx>,
    /// The instruction after which to add the required cleanup instructions.
    pub close_after_instruction: InstructionValue<'ctx>,
    /// The function context snapshot to be set when writing cleanup instructions.
    pub snapshot: Snapshot<'ctx>,
}

/// During the reverse continuation phase, when closing this continuation block,
/// insert the instructions to build the closure after this instruction, and
/// replace this instruction with the allocation of a properly sized closure array.
#[derive(Clone, Debug)]
pub(super) enum Exit<'ctx> {
    Close(Parent<'ctx>),
    Clean(Parent<'ctx>),
    Capture(Parent<'ctx>),
}

impl<'ctx> Exit<'ctx> {
    fn parent(&self) -> &Parent<'ctx> {
        match self {
            Self::Close(parent) | Self::Clean(parent) | Self::Capture(parent) => parent,
        }
    }
}

/// A `Merger` collects ended continuation points without starting the next continuation point immediately,
/// typically when there are multiple continuations being built separately which will later both continue
/// to the same place (i.e. merge).
#[derive(Default)]
pub(crate) struct Merger<'ctx>(Vec<Exit<'ctx>>);

impl<'ctx> Merger<'ctx> {
    fn close_from(
        &mut self,
        parent: &Rc<ContinuationPoint<'ctx>>,
        instruction: InstructionValue<'ctx>,
        snapshot: Snapshot<'ctx>,
    ) {
        self.0.push(Exit::Close(Parent {
            parent: Rc::downgrade(parent),
            closure_instruction: instruction,
            close_after_instruction: instruction,
            snapshot,
        }));
    }
}

static CONTINUATION_POINT_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A continuation point tracks the values in scope and in closure for any segment of code
/// that resides within one unbroken continuation. At the LLVM level, this can be considered
/// one function; a single Trilogy function may be made up of numerous LLVM functions, its
/// "continuation points".
///
/// The continuation points form a visibility chain based on the lexical structure of the
/// program, as well as a control-flow graph based on the semantic structure of the program.
/// These two structures do not have to align perfectly: a closure is semantically disconnected
/// but lexically connected, while a merge is lexically disconnected but semantically connected.
#[derive(Clone, Debug)]
pub(crate) struct ContinuationPoint<'ctx> {
    #[allow(
        dead_code,
        reason = "handy debugging thing, maybe remove when the bugs are all gone"
    )]
    pub id: usize,
    /// Pointers to variables available at this point in the continuation.
    /// These pointers may be to values on stack, or to locations in the closure.
    pub(super) variables: RefCell<HashMap<Closed<'ctx>, Variable<'ctx>>>,
    /// The list of all variables which can possibly be referenced from this location.
    /// If the variable is not already referenced (i.e. found in the `variables` map),
    /// then it must be requested from all of the `capture_from` continuation points
    /// and added to the closure array and variables map.
    pub(super) parent_variables: HashSet<Closed<'ctx>>,

    /// The module closure must be implicitly included in the closure array, and carried
    /// so long as control remains within the scope of the module.
    pub(super) module_closure: Vec<Id>,

    /// Maintains the order of variables found in the closure array.
    pub(super) closure: RefCell<Vec<Closed<'ctx>>>,
    /// The lexical pre-continuations from which this continuation may be reached. May be many
    /// in the case of branching instructions such as `if` or `match`.
    pub(super) parents: Vec<Exit<'ctx>>,
    pub(super) shadows: Option<Weak<ContinuationPoint<'ctx>>>,
}

impl<'ctx> ContinuationPoint<'ctx> {
    pub(crate) fn new(module_closure: Vec<Id>) -> Self {
        Self {
            id: CONTINUATION_POINT_COUNTER.fetch_add(1, Ordering::Relaxed),
            variables: RefCell::default(),
            closure: RefCell::new(
                module_closure
                    .iter()
                    .map(|id| Closed::Variable(id.clone()))
                    .collect(),
            ),
            parent_variables: module_closure
                .iter()
                .map(|id| Closed::Variable(id.clone()))
                .collect(),
            module_closure,
            parents: vec![],
            shadows: None,
        }
    }

    fn compute_parent_variables(&self) -> HashSet<Closed<'ctx>> {
        let mut variables: HashSet<_> = self
            .variables
            .borrow()
            .keys()
            .chain(self.parent_variables.iter())
            .cloned()
            .collect();
        if let Some(shadowed) = &self.shadows {
            variables.extend(shadowed.upgrade().unwrap().compute_parent_variables());
        }
        variables
    }

    /// Creates a new continuation point which has visibility of the current one's variables.
    fn chain(self: &Rc<Self>) -> Self {
        Self {
            id: CONTINUATION_POINT_COUNTER.fetch_add(1, Ordering::Relaxed),
            variables: RefCell::default(),
            closure: RefCell::new(
                self.module_closure
                    .iter()
                    .map(|id| Closed::Variable(id.clone()))
                    .collect(),
            ),
            module_closure: self.module_closure.clone(),
            parent_variables: self.compute_parent_variables(),
            parents: vec![],
            shadows: None,
        }
    }

    /// Creates a new continuation point which is "the same" as the current one, but is extended
    /// with variables independently (typically due to branching).
    fn shadow(self: &Rc<Self>) -> Self {
        Self {
            id: CONTINUATION_POINT_COUNTER.fetch_add(1, Ordering::Relaxed),
            variables: RefCell::default(), // self.variables.clone(),
            closure: RefCell::default(),
            module_closure: self.module_closure.clone(),
            parent_variables: HashSet::default(), // self.parent_variables.clone(),
            parents: vec![],
            shadows: Some(Rc::downgrade(self)),
        }
    }

    fn close_from(
        &mut self,
        parent: &Rc<ContinuationPoint<'ctx>>,
        closure_instruction: InstructionValue<'ctx>,
        snapshot: Snapshot<'ctx>,
    ) {
        self.parents.push(Exit::Close(Parent {
            parent: Rc::downgrade(parent),
            closure_instruction,
            close_after_instruction: closure_instruction,
            snapshot,
        }));
    }

    fn close_from_after(
        &mut self,
        parent: &Rc<ContinuationPoint<'ctx>>,
        closure_instruction: InstructionValue<'ctx>,
        close_after_instruction: InstructionValue<'ctx>,
        snapshot: Snapshot<'ctx>,
    ) {
        self.parents.push(Exit::Close(Parent {
            parent: Rc::downgrade(parent),
            closure_instruction,
            close_after_instruction,
            snapshot,
        }));
    }

    fn clean_from(
        &mut self,
        parent: &Rc<ContinuationPoint<'ctx>>,
        instruction: InstructionValue<'ctx>,
        snapshot: Snapshot<'ctx>,
    ) {
        self.parents.push(Exit::Clean(Parent {
            parent: Rc::downgrade(parent),
            closure_instruction: instruction,
            close_after_instruction: instruction,
            snapshot,
        }));
    }

    fn capture_from(
        &mut self,
        parent: &Rc<ContinuationPoint<'ctx>>,
        instruction: InstructionValue<'ctx>,
        snapshot: Snapshot<'ctx>,
    ) {
        self.parents.push(Exit::Capture(Parent {
            parent: Rc::downgrade(parent),
            closure_instruction: instruction,
            close_after_instruction: instruction,
            snapshot,
        }));
    }

    pub(crate) fn contains_temporary(&self, temporary: PointerValue<'ctx>) -> bool {
        if let Some(shadowed) = &self.shadows
            && shadowed.upgrade().unwrap().contains_temporary(temporary)
        {
            return true;
        }
        let key = Closed::Temporary(temporary);
        self.parent_variables.contains(&key) || self.variables.borrow().contains_key(&key)
    }

    pub(crate) fn shadow_root(self: &Rc<ContinuationPoint<'ctx>>) -> Rc<ContinuationPoint<'ctx>> {
        match &self.shadows {
            Some(shadowed) => shadowed.upgrade().unwrap().shadow_root(),
            None => self.clone(),
        }
    }
}

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn current_continuation_point(&self) -> Rc<ContinuationPoint<'ctx>> {
        self.continuation_points.borrow().last().unwrap().clone()
    }

    /// Ends the current continuation point. Cleanup code will be inserted before the provided
    /// instruction which captures any values referenced by the continuation, and destroys any
    /// remaining values that are going out of scope. The instruction should be an `alloca`
    /// used as if it were the closure value, and will be replaced with the actual closure
    /// construction instructions at a later time.
    ///
    /// A new implicit continuation point is started immediately.
    pub(crate) fn end_continuation_point_as_close(
        &self,
        closure_allocation: InstructionValue<'ctx>,
    ) {
        let mut cps = self.continuation_points.borrow_mut();
        let last = cps.last().unwrap();
        let mut next = last.chain();
        next.close_from(last, closure_allocation, self.snapshot_function_context());
        cps.push(Rc::new(next));
    }

    pub(crate) fn end_continuation_point_as_close_after(
        &self,
        closure_allocation: InstructionValue<'ctx>,
        close_after_instruction: InstructionValue<'ctx>,
    ) {
        let mut cps = self.continuation_points.borrow_mut();
        let last = cps.last().unwrap();
        let mut next = last.chain();
        next.close_from_after(
            last,
            closure_allocation,
            close_after_instruction,
            self.snapshot_function_context(),
        );
        cps.push(Rc::new(next));
    }

    /// Ends the current continuation point. Cleanup code will be inserted before the provided
    /// instruction which destroys all values that will be going out of scope. The instruction
    /// should typically be the final `call` instruction in the that is being exited.
    ///
    /// After this, there is no valid implicit continuation point. A cleaned continuation should
    /// be at the end of a lexical scope.
    pub(crate) fn end_continuation_point_as_clean(&self, call_instruction: InstructionValue<'ctx>) {
        let mut cps = self.continuation_points.borrow_mut();
        let last = cps.last().unwrap();
        let mut next = last.chain();
        next.clean_from(last, call_instruction, self.snapshot_function_context());
        cps.push(Rc::new(next));
    }

    /// Ends the current continuation point. Cleanup code will be inserted before the provided
    /// instruction which destroys all values that will be going out of scope. The instruction
    /// should typically be the final `call` instruction in the that is being exited.
    ///
    /// After this, there is no valid implicit continuation point. A cleaned continuation should
    /// be at the end of a lexical scope.
    pub(crate) fn capture_contination_point(
        &self,
        closure_allocation: InstructionValue<'ctx>,
    ) -> Rc<ContinuationPoint<'ctx>> {
        let cps = self.continuation_points.borrow();
        let current = cps.last().unwrap();
        let mut next = current.chain();
        next.capture_from(
            current,
            closure_allocation,
            self.snapshot_function_context(),
        );
        Rc::new(next)
    }

    /// Reinstates a previously held continuation point.
    pub(crate) fn become_continuation_point(
        &self,
        continuation_point: Rc<ContinuationPoint<'ctx>>,
    ) {
        self.continuation_points
            .borrow_mut()
            .push(continuation_point);
    }

    /// Prepares the current continuation point to branch into multiple endings.
    /// The current continuation point remains the implicit continuation point,
    /// but can safely be ignored without ending it, as it may be ended later.
    pub(crate) fn branch_continuation_point(&self) -> Rc<ContinuationPoint<'ctx>> {
        let mut cps = self.continuation_points.borrow_mut();
        let current = cps.last().unwrap();
        let now = Rc::new(current.shadow());
        let later = Rc::new(current.shadow());
        cps.push(now);
        later
    }

    /// Creates a new, unattached, shadow of the current continuation point.
    pub(crate) fn shadow_continuation_point(&self) -> Rc<ContinuationPoint<'ctx>> {
        let cps = self.continuation_points.borrow();
        let current = cps.last().unwrap();
        Rc::new(current.shadow())
    }

    /// Ends the current continuation point, but does not start a new implicit
    /// continuation point.
    pub(crate) fn end_continuation_point_as_merge(
        &self,
        merger: &mut Merger<'ctx>,
        closure_allocation: InstructionValue<'ctx>,
    ) {
        let cps = self.continuation_points.borrow();
        merger.close_from(
            cps.last().unwrap(),
            closure_allocation,
            self.snapshot_function_context(),
        );
    }

    /// Ties a Merger's collected exits to a new continuation point with visibility to
    /// the values of one of the branches, arbitrarily.
    ///
    /// Technically, since valid variable references have already been resolved at the
    /// IR level, this additional visibility is not of danger.
    ///
    /// There should not be a current implicit continuation point when calling this. A
    /// new one is set afterwards.
    pub(crate) fn merge_without_branch(&self, merger: Merger<'ctx>) {
        let mut cps = self.continuation_points.borrow_mut();
        let mut cp = merger
            .0
            .first()
            .unwrap()
            .parent()
            .parent
            .upgrade()
            .unwrap()
            .chain();
        cp.parents = merger.0;
        cps.push(Rc::new(cp))
    }
}
