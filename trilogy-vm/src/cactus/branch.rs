use super::{Cactus, Slice};

/// A branch of a Cactus stack.
///
/// A branch is a regular stack which may be "attached" to a shared portion of a root Cactus.
#[derive(Clone, Debug)]
pub struct Branch<'a, T> {
    cactus: Slice<'a, T>,
    stack: Vec<T>,
    len: usize,
}

impl<'a, T> Branch<'a, T> {
    pub(super) fn new(cactus: &'a Cactus<T>) -> Self {
        Self {
            cactus: Slice::new(cactus),
            stack: vec![],
            len: 0,
        }
    }

    #[inline]
    pub fn cactus(&self) -> &'a Cactus<T> {
        self.cactus.cactus()
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
        let value = self.cactus.pop();
        if value.is_some() {
            self.len -= 1;
        }
        value
    }

    /// Pops multiple values off the stack.
    ///
    /// The returned items are popped in their original order, opposite the order they would be
    /// after popping n times separately and pushing into a new vec.
    pub fn pop_n(&mut self, n: usize) -> Vec<T>
    where
        T: Clone,
    {
        self.consume_to_length(n);
        self.stack.drain(self.stack.len() - n..).collect()
    }

    fn consume_to_length(&mut self, length: usize)
    where
        T: Clone,
    {
        if self.stack.len() < length {
            let mut popped = self.cactus.pop_n(length - self.stack.len());
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
        if index >= self.cactus.len() {
            Some(self.stack[index - self.cactus.len()].clone())
        } else {
            self.cactus.get(index)
        }
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: T) {
        if index >= self.cactus.len() {
            self.stack[index - self.cactus.len()] = value;
        } else {
            self.cactus.set(index, value);
        }
    }

    /// Moves the values from this stack's local branch into the shared parent.
    ///
    /// Future clones of this branch will share those elements. Previously existing
    /// clones will remain distinct.
    #[inline]
    pub fn commit(&mut self) -> Slice<'a, T> {
        self.cactus.append(&mut self.stack);
        self.cactus.clone()
    }

    /// Branches the current branch into two, being self as one, and returning the other.
    ///
    /// All elements in the current branch are moved to the shared base, and both branches
    /// will have the same shared parents.
    pub fn branch(&mut self) -> Self {
        let cactus = self.commit();
        Self {
            cactus,
            stack: vec![],
            len: self.len,
        }
    }
}

impl<'a, T> From<Slice<'a, T>> for Branch<'a, T> {
    fn from(value: Slice<'a, T>) -> Self {
        Self {
            len: value.len(),
            cactus: value,
            stack: vec![],
        }
    }
}
