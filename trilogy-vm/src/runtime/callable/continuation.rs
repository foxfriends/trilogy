use super::super::RefCount;
use crate::bytecode::Offset;
use crate::cactus::{Branch, Pointer, Slice};
use crate::gc::Dumpster;
use crate::vm::stack::{Cont, Stack, StackCell, StackFrame};
use std::fmt::{self, Debug};
use std::hash::Hash;
use std::sync::Weak;

/// A continuation from a Trilogy program.
///
/// From within the program this is seen as an opaque "callable" value.
///
/// It is not possible to construct a value of this type except from within a
/// Trilogy program.
#[derive(Clone)]
pub(crate) struct Continuation(RefCount<InnerContinuation>);

impl Debug for Continuation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Continuation")
            .field("ip", &self.0.ip)
            .field("frames", &self.0.frames)
            .field("branch", &self.0.branch)
            .finish()
    }
}

#[derive(Clone, Debug)]
struct FramePointer {
    stack: Option<Pointer<StackCell>>,
    cont: Cont,
    fp: usize,
}

#[derive(Clone)]
struct InnerContinuation {
    ip: Offset,
    dumpster: Weak<Dumpster>,
    frames: Vec<FramePointer>,
    branch: Pointer<StackCell>,
    fp: usize,
}

impl InnerContinuation {
    #[inline(always)]
    fn new(ip: Offset, dumpster: Weak<Dumpster>, stack: Stack<'_>) -> Self {
        log::debug!("allocating continuation");
        #[cfg(feature = "stats")]
        crate::GLOBAL_STATS
            .continuations_allocated
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let (frames, mut branch, fp) = stack.into_parts();
        branch.commit();
        let branch = branch.slice().clone().into_pointer();
        Self {
            ip,
            dumpster,
            frames: frames
                .into_iter()
                .map(|frame| FramePointer {
                    stack: frame.slice.map(|cactus| cactus.into_pointer()),
                    cont: frame.cont.clone(),
                    fp: frame.fp,
                })
                .collect(),
            branch,
            fp,
        }
    }
}

impl Drop for InnerContinuation {
    fn drop(&mut self) {
        // if let Some(dumpster) = self.dumpster.upgrade() {
        //     for frame in &self.frames {
        //         if let Some(stack) = &frame.stack {
        //             dumpster.throw_out(stack.ranges())
        //         }
        //     }
        //     dumpster.throw_out(self.branch.ranges());
        // }
        #[cfg(feature = "stats")]
        crate::GLOBAL_STATS
            .continuations_freed
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Eq for Continuation {}

impl PartialEq for Continuation {
    fn eq(&self, other: &Self) -> bool {
        RefCount::ptr_eq(&self.0, &other.0)
    }
}

impl Hash for Continuation {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        RefCount::as_ptr(&self.0).hash(state);
    }
}

impl Continuation {
    #[inline(always)]
    pub(crate) fn new(ip: Offset, dumpster: Weak<Dumpster>, stack: Stack<'_>) -> Self {
        Self(RefCount::new(InnerContinuation::new(ip, dumpster, stack)))
    }

    #[inline(always)]
    pub(crate) fn ip(&self) -> Offset {
        self.0.ip
    }

    #[inline(always)]
    pub(crate) unsafe fn stack<'a>(&self) -> Stack<'a> {
        let frames = self
            .0
            .frames
            .iter()
            .map(|frame| {
                let stack = frame.stack.as_ref().map(|frame| unsafe {
                    let pointer = frame.clone();
                    pointer.reacquire();
                    Slice::from_pointer(pointer)
                });
                StackFrame {
                    slice: stack,
                    cont: frame.cont.clone(),
                    fp: frame.fp,
                }
            })
            .collect();
        let branch = unsafe {
            let pointer = self.0.branch.clone();
            pointer.reacquire();
            Branch::from(Slice::from_pointer(pointer))
        };
        Stack::from_parts(frames, branch, self.0.dumpster.clone(), self.0.fp)
    }
}
