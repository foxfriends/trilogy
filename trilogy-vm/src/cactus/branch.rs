use super::{Cactus, Slice};

/// A branch of a Cactus stack.
///
/// A branch is a regular stack which may be "attached" to a shared portion of a root Cactus.
#[derive(Clone, Debug)]
pub struct Branch<'a, T> {
    slice: Slice<'a, T>,
    stack: Vec<T>,
    len: usize,
}

impl<'a, T> Branch<'a, T> {
    pub(super) fn new(cactus: &'a Cactus<T>) -> Self {
        Self {
            slice: Slice::new(cactus),
            stack: vec![],
            len: 0,
        }
    }

    #[inline]
    pub fn cactus(&self) -> &'a Cactus<T> {
        self.slice.cactus()
    }

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

    /// Peeks a locally owned value from this stack.
    ///
    /// This will never peek at a value from the parent stack.
    #[inline]
    pub fn peek_local(&self) -> Option<&T> {
        self.stack.last()
    }

    /// A reference to the shared slice that backs this Branch. The slice does not
    /// include any of the local elements in this branch.
    #[inline]
    pub fn slice(&self) -> &Slice<'a, T> {
        &self.slice
    }

    /// Consumes this branch, discarding its local segment, and returning the backing shared
    /// slice.
    #[inline]
    pub fn into_slice(self) -> Slice<'a, T> {
        self.slice
    }

    /// Pops a value off this stack. If there are values in the local stack, those will
    /// be popped first, otherwise a cloned value from the shared stack is "popped" from
    /// this branch's view of its parents.
    #[inline]
    pub fn pop(&mut self) -> Option<T>
    where
        T: Clone,
    {
        let popped = self.pop_local();
        if popped.is_some() {
            return popped;
        }
        let value = self.slice.pop();
        if value.is_some() {
            self.len -= 1;
        }
        value
    }

    /// Peeks at the last value on this stack without popping it.
    #[inline]
    pub fn peek(&mut self) -> Option<T>
    where
        T: Clone,
    {
        let peeked = self.peek_local();
        if peeked.is_some() {
            return peeked.cloned();
        }
        self.slice.peek()
    }

    /// Pops multiple values off the stack.
    ///
    /// The returned items are popped in their original order, opposite the order they would be
    /// after popping n times separately and pushing into a new vec.
    ///
    /// If there are not enough values, returns None without attempting to pop any.
    #[inline]
    pub fn pop_n(&mut self, n: usize) -> Option<Vec<T>>
    where
        T: Clone,
    {
        if self.len < n {
            return None;
        }
        self.consume_to_length(n);
        let elements: Vec<_> = self.stack.drain(self.stack.len() - n..).collect();
        self.len -= n;
        Some(elements)
    }

    /// Un-shares elements from the parent until this branch's local length is at least
    /// `length`.
    ///
    /// If there are not enough elements, the branch's local length may still be less
    /// than desired.
    #[inline]
    pub fn consume_to_length(&mut self, length: usize)
    where
        T: Clone,
    {
        if self.stack.len() < length {
            let mut popped = self
                .slice
                .pop_n(usize::min(self.slice.len(), length - self.stack.len()))
                .unwrap();
            popped.append(&mut self.stack);
            self.stack = popped;
        }
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        self.stack.push(value);
        self.len += 1;
    }

    #[inline]
    pub fn append(&mut self, values: &mut Vec<T>) {
        self.len += values.len();
        self.stack.append(values);
    }

    #[inline]
    pub fn local_len(&self) -> usize {
        self.stack.len()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.stack.capacity()
    }

    #[inline]
    pub fn reserve(&mut self, count: usize) {
        self.stack.reserve(count);
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        if self.slice.len() <= len {
            self.stack.truncate(len - self.slice.len());
        } else {
            self.slice.truncate(len);
            self.stack.clear();
        }
        self.len = self.slice.len() + self.stack.len();
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
    pub fn get(&self, index: usize) -> Option<T>
    where
        T: Clone,
    {
        if index >= self.slice.len() {
            if index - self.slice.len() < self.stack.len() {
                Some(self.stack[index - self.slice.len()].clone())
            } else {
                None
            }
        } else {
            self.slice.get(index)
        }
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: T) {
        if index >= self.slice.len() {
            self.stack[index - self.slice.len()] = value;
        } else {
            self.slice.set(index, value);
        }
    }

    /// Moves the values from this stack's local branch into the shared parent.
    ///
    /// Future clones of this branch will share those elements. Previously existing
    /// clones will remain distinct.
    #[inline]
    pub fn commit(&mut self) {
        self.slice.append(&mut self.stack);
    }

    /// Branches the current branch into two, being self as one, and returning the other.
    ///
    /// All elements in the current branch are moved to the shared base, and both branches
    /// will have the same shared parents.
    pub fn branch(&mut self) -> Self {
        self.commit();
        Self {
            slice: self.slice.clone(),
            stack: vec![],
            len: self.len,
        }
    }

    pub fn iter<'b>(&'b self) -> BranchIter<'a, 'b, T> {
        BranchIter {
            branch: self,
            index: 0,
        }
    }
}

impl<'a, T> From<Slice<'a, T>> for Branch<'a, T> {
    fn from(slice: Slice<'a, T>) -> Self {
        Self {
            len: slice.len(),
            slice,
            stack: vec![],
        }
    }
}

pub struct BranchIter<'a, 'b, T> {
    branch: &'b Branch<'a, T>,
    index: usize,
}

impl<'a, 'b, T> Iterator for BranchIter<'a, 'b, T>
where
    T: Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.branch.get(self.index);
        self.index += 1;
        val
    }
}
