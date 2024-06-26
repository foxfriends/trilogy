use super::super::RefCount;
use crate::bytecode::Offset;
use crate::cactus::{Branch, Pointer, Slice, StackOverflow};
use crate::vm::stack::{Cont, Stack, StackCell, StackFrame};
use std::fmt::{self, Debug};
use std::hash::Hash;

/// A continuation from a Trilogy program.
///
/// From within the program this is seen as an opaque "callable" value.
///
/// It is not possible to construct a value of this type except from within a
/// Trilogy program. If held beyond the end of the execution of the Trilogy
/// program, it is no longer valid to be called.
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
    #[inline]
    pub(crate) fn new(ip: Offset, stack: Stack<'_>) -> Result<Self, StackOverflow> {
        Ok(Self(RefCount::new(InnerContinuation::new(ip, stack)?)))
    }

    /// Returns the ID of the underlying continuation instance. This ID will remain
    /// stable for the lifetime of each array instance, and is unique per
    /// instance.
    #[inline]
    pub fn id(&self) -> usize {
        RefCount::as_ptr(&self.0) as usize
    }

    #[inline]
    pub fn ip(&self) -> Offset {
        self.0.ip
    }

    #[inline]
    pub fn stack_pointer(&self) -> &Pointer<StackCell> {
        &self.0.branch
    }

    #[inline]
    pub fn frames(&self) -> impl Iterator<Item = &Pointer<StackCell>> {
        self.0
            .frames
            .iter()
            .filter_map(|frame| frame.stack.as_ref())
    }

    #[inline]
    pub(crate) unsafe fn stack<'a>(&self) -> Stack<'a> {
        let frames = self
            .0
            .frames
            .iter()
            .map(|frame| {
                let stack = frame
                    .stack
                    .as_ref()
                    .map(|frame| unsafe { Slice::from_pointer(frame.clone()) });
                StackFrame {
                    slice: stack,
                    cont: frame.cont.clone(),
                    fp: frame.fp,
                    here: frame.here,
                }
            })
            .collect();
        let branch = unsafe { Branch::from(Slice::from_pointer(self.0.branch.clone())) };
        Stack::from_parts(frames, branch, self.0.fp)
    }
}

#[derive(Debug)]
struct FramePointer {
    stack: Option<Pointer<StackCell>>,
    cont: Cont,
    fp: usize,
    here: Option<Offset>,
}

struct InnerContinuation {
    ip: Offset,
    frames: Vec<FramePointer>,
    branch: Pointer<StackCell>,
    fp: usize,
}

impl InnerContinuation {
    #[inline]
    fn new(ip: Offset, stack: Stack<'_>) -> Result<Self, StackOverflow> {
        log::debug!("allocating continuation");
        #[cfg(feature = "stats")]
        crate::GLOBAL_STATS
            .continuations_allocated
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let (frames, mut branch, fp) = stack.into_parts();
        branch.commit()?;
        let branch = branch.into_slice().into_pointer();
        Ok(Self {
            ip,
            frames: frames
                .into_iter()
                .map(|frame| FramePointer {
                    stack: frame.slice.map(|cactus| cactus.into_pointer()),
                    cont: frame.cont.clone(),
                    fp: frame.fp,
                    here: frame.here,
                })
                .collect(),
            branch,
            fp,
        })
    }
}

#[cfg(feature = "stats")]
impl Drop for InnerContinuation {
    fn drop(&mut self) {
        crate::GLOBAL_STATS
            .continuations_freed
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}
