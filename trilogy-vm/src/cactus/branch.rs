use super::Cactus;
use std::ops::Range;

/// A branch of a Cactus stack.
///
/// A branch is a regular stack which may be "attached" to a shared portion of a root Cactus.
pub struct Branch<'a, T> {
    cactus: &'a Cactus<T>,
    parents: Vec<Range<usize>>,
    stack: Vec<T>,
    len: usize,
}

impl<'a, T> Drop for Branch<'a, T> {
    fn drop(&mut self) {
        for range in self.parents {
            self.cactus.release_range(range);
        }
    }
}

impl<T: Clone> Clone for Branch<'_, T> {
    fn clone(&self) -> Self {
        let ranges = self.cactus.ranges.lock().unwrap();
        for range in self.parents {
            self.cactus.acquire_range(range);
        }
        Self {
            cactus: self.cactus,
            parents: self.parents.clone(),
            stack: self.stack.clone(),
            len: self.len,
        }
    }
}

impl<T> Branch<'_, T> {
    /// Pops a locally owned value off this branch.
    ///
    /// This will never pop a value off a shared parent.
    #[inline]
    pub fn pop_local(&mut self) -> Option<T> {
        let value = self.stack.pop();
        if value.is_some() {
            self.len -= 1;
        }
        value
    }

    /// Pops a value off this stack. If there are values in the local stack, those will
    /// be popped first, otherwise a cloned value from the shared stack is "popped" from
    /// this branch's view of its parents.
    pub fn pop(&mut self) -> Option<T>
    where
        T: Clone,
    {
        let popped = self.pop_local();
        if popped.is_some() {
            return popped;
        }
        let mut parent = self.parents.last_mut()?;
        let index = parent.end;
        let value = unsafe {
            self.cactus.stack.lock().unwrap()[index]
                .assume_init_ref()
                .clone()
        };
        self.cactus.release(index);
        parent.end -= 1;
        if parent.end == parent.start {
            self.parents.pop();
        }
        self.len -= 1;
        Some(value)
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        self.stack.push(value);
        self.len += 1;
    }

    #[inline]
    pub fn local_len(&self) -> usize {
        self.stack.len()
    }

    #[inline]
    pub fn local_capacity(&self) -> usize {
        self.stack.capacity()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    fn resolve_index(&self, mut index: usize) -> Option<usize> {
        for range in &self.parents {
            if range.len() > index {
                return Some(range.start + index);
            }
            index -= range.len();
        }
        None
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<T>
    where
        T: Clone,
    {
        if index >= self.len - self.stack.len() {
            Some(self.stack[index].clone())
        } else {
            let parent_index = self.resolve_index(index)?;
            unsafe { self.cactus.get_unchecked(parent_index) }
        }
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: T) {
        if index >= self.len - self.stack.len() {
            self.stack[index] = value;
        } else {
            let parent_index = self.resolve_index(index).unwrap();
            unsafe {
                self.cactus.set_unchecked(parent_index, value);
            }
        }
    }

    /// Moves the values from this stack's local branch into the shared parent.
    ///
    /// Future clones of this branch will share those elements. Previously existing
    /// clones will remain distinct.
    pub fn commit(&mut self) {
        let range = self.cactus.append(&mut self.stack);
        self.parents.push(range);
    }
}
