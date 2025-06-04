//! Functions for managing context and scope across continuation points.

use super::{Closed, Codegen, Snapshot, Variable};
use inkwell::debug_info::DILocation;
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
    /// The instruction around which to add the required cleanup instructions. The exact
    /// interpretation of this instruction depends on the variant of `Exit` this is
    /// contained in.
    pub instruction: InstructionValue<'ctx>,
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

/// A `Brancher` is created when a continuation point will end in more than one place (i.e. it branches).
///
/// At each place this continuation point might end, an `add_branch_end_` function should be called
/// to correctly end the continuation in that position.
pub(crate) struct Brancher<'ctx>(Rc<ContinuationPoint<'ctx>>);

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
            instruction,
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
    /// The mapping from variable names to their upvalues. If one already exists for a variable
    /// as it is being captured, it must be reused.
    pub(super) upvalues: RefCell<HashMap<Closed<'ctx>, PointerValue<'ctx>>>,
    /// The lexical pre-continuations from which this continuation may be reached. May be many
    /// in the case of branching instructions such as `if` or `match`.
    pub(super) parents: Vec<Exit<'ctx>>,

    /// A bit of a hack, but this is tracking all places that a variable is destroyed during
    /// scope cleanup without being closed. If we later determine that we need to close that
    /// variable, this allows us to go back and make sure it was closed after all.
    pub(super) unclosed:
        RefCell<HashMap<PointerValue<'ctx>, Vec<(InstructionValue<'ctx>, DILocation<'ctx>)>>>,
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
            upvalues: RefCell::default(),
            parents: vec![],
            unclosed: RefCell::default(),
        }
    }

    /// Creates a new continuation point which has visibility of the current one's variables.
    fn chain(&self) -> Self {
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
        snapshot: Snapshot<'ctx>,
    ) {
        self.parents.push(Exit::Close(Parent {
            parent: Rc::downgrade(parent),
            instruction,
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
            instruction,
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
            instruction,
            snapshot,
        }));
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

    /// Pops the current continuation point, typically because it is not yet being handled.
    /// Add it back later with `become_continuation_point` when it is ready to be written to.
    pub(crate) fn hold_continuation_point(&self) -> Rc<ContinuationPoint<'ctx>> {
        self.continuation_points.borrow_mut().pop().unwrap()
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

    /// Adds an ending to the previously branched continuation point. There should be no
    /// existing implicit continuation point, and starts a new implicit continuation point.
    pub(crate) fn add_branch_end_as_close(
        &self,
        brancher: &Brancher<'ctx>,
        closure_allocation: InstructionValue<'ctx>,
    ) {
        let mut cps = self.continuation_points.borrow_mut();
        let mut next = brancher.0.chain();
        next.close_from(
            &brancher.0,
            closure_allocation,
            self.snapshot_function_context(),
        );
        cps.push(Rc::new(next));
    }

    /// Resumes a continuation point that was previously held for a branch, which was used
    /// only for non-ending captures, and not actually ended. The previously branched from
    /// continuation point becomes the implicit continuation point again.
    pub(crate) fn resume_continuation_point(&self, brancher: &Brancher<'ctx>) {
        let mut cps = self.continuation_points.borrow_mut();
        cps.push(brancher.0.clone());
    }

    /// Adds a non-ending capture on this branch. The branch must still be ended later with
    /// a proper branch ending function.
    ///
    /// A placeholder closure allocation must be passed, which will later be replaced by
    /// instructions to capture the values required by the closure.
    ///
    /// There should be no implicit continuation point before calling this, as this will
    /// start a new one.
    pub(crate) fn add_branch_capture(
        &self,
        brancher: &Brancher<'ctx>,
        closure_allocation: InstructionValue<'ctx>,
    ) {
        let mut cps = self.continuation_points.borrow_mut();
        let mut next = brancher.0.chain();
        next.capture_from(
            &brancher.0,
            closure_allocation,
            self.snapshot_function_context(),
        );
        cps.push(Rc::new(next));
    }

    /// Prepares the current continuation point to branch into multiple endings.
    /// The current continuation point remains the implicit continuation point,
    /// but can safely be ignored without ending it, as it may be ended later.
    pub(crate) fn end_continuation_point_as_branch(&self) -> Brancher<'ctx> {
        let parent = self.continuation_points.borrow().last().unwrap().clone();
        Brancher(parent)
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

    /// Ties a Merger's collected exits to a new continuation point, with visibility to
    /// only their shared parent Brancher's variables in scope.
    ///
    /// There should not be a current implicit continuation point when calling this. A
    /// new one is set afterwards.
    pub(crate) fn merge_branch(&self, branch: Brancher<'ctx>, merger: Merger<'ctx>) {
        let mut cps = self.continuation_points.borrow_mut();
        let mut cp = branch.0.chain();
        cp.parents = merger.0;
        cps.push(Rc::new(cp))
    }

    /// Ties a Merger's collected exits to a new continuation point with visibility to
    /// the values of one of the branches, arbitrarily.
    ///
    /// Technically, since valid variable references have already been resolved at the
    /// IR level, this additional visibility is not of danger, so we could use this
    /// always instead of `merge_branch`, but... merge branch is a bit more intuitive.
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
