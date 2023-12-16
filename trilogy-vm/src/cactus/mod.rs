//! A cactus stack.
//!
//! This is the stack implementation that backs the Trilogy VM, where branches
//! are used to represent continuations and closures that share a parent stack
//! but have differing active stacks.
use rangemap::RangeMap;
use std::mem::MaybeUninit;
use std::ops::Range;
use std::sync::{Arc, Mutex};

mod branch;
mod slice;

pub use branch::Branch;
pub use slice::Slice;

/// The root of the Cactus Stack.
///
/// The actual stack itself is accessed through `Branch`es.
pub struct Cactus<T> {
    /// The backing memory of this stack. This space is sparse, so accessing values directly
    /// is unsafe, as not every cell of the Vec may be initialized.
    stack: Mutex<Vec<MaybeUninit<T>>>,
    /// Reference counts for each range in the stack. When a range reaches 0 references,
    /// its elements should be uninitialized. It is only safe to access values where
    /// the reference count for its index is non-zero.
    ranges: Arc<Mutex<RangeMap<usize, usize>>>,
}

impl<T> Default for Cactus<T> {
    #[inline]
    fn default() -> Self {
        Self {
            stack: Default::default(),
            ranges: Default::default(),
        }
    }
}

impl<T> Cactus<T> {
    /// Creates a new empty cactus.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::Cactus;
    /// let cactus = Cactus::<usize>::new();
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Initializes a cactus with a specific capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::Cactus;
    /// let cactus = Cactus::<usize>::with_capacity(10);
    /// assert!(cactus.capacity() >= 10);
    /// ```
    #[inline(always)]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            stack: Mutex::new(Vec::with_capacity(cap)),
            ranges: Default::default(),
        }
    }

    /// Returns the total number of elements this Cactus can hold without reallocating.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.stack.lock().unwrap().capacity()
    }

    /// Reserves capacity for at least `additional` more elements to be added to this Cactus.
    #[inline(always)]
    pub fn reserve(&self, additional: usize) {
        self.stack.lock().unwrap().reserve(additional);
    }

    /// Returns a new branch on this cactus. This branch contains no elements (i.e. comes
    /// straight out from the ground).
    #[inline(always)]
    pub fn branch(&self) -> Branch<T> {
        Branch {
            cactus: self,
            parents: vec![],
            stack: vec![],
            len: 0,
        }
    }

    unsafe fn get_unchecked(&self, index: usize) -> Option<T>
    where
        T: Clone,
    {
        self.stack
            .lock()
            .unwrap()
            .get(index)
            .map(|val| val.assume_init_ref().clone())
    }

    unsafe fn set_unchecked(&self, index: usize, value: T) {
        *self.stack.lock().unwrap()[index].assume_init_mut() = value;
    }

    #[inline]
    fn acquire_range(&self, range: Range<usize>) {
        let ranges = self.ranges.lock().unwrap();
        for (&subrange, &value) in ranges.overlapping(&range) {
            let subrange =
                usize::max(subrange.start, range.start)..usize::min(subrange.end, range.end);
            ranges.insert(subrange, value + 1);
        }
    }

    #[inline(always)]
    fn release(&self, index: usize) {
        self.release_range(index..index + 1);
    }

    #[inline(always)]
    fn append(&self, values: &mut Vec<T>) -> Range<usize> {
        let ranges = self.ranges.lock().unwrap();
        let stack = self.stack.lock().unwrap();
        let len = values.len();
        let range = stack.len()..stack.len() + len;

        stack.extend(values.drain(..).map(MaybeUninit::new));

        // Acquire range, but we already locked it:
        for (&subrange, &value) in ranges.overlapping(&range) {
            let subrange =
                usize::max(subrange.start, range.start)..usize::min(subrange.end, range.end);
            ranges.insert(subrange, value + 1);
        }

        range
    }

    #[inline]
    fn release_range(&self, range: Range<usize>) {
        let ranges = self.ranges.lock().unwrap();
        for (&subrange, &value) in ranges.overlapping(&range) {
            let subrange =
                usize::max(subrange.start, range.start)..usize::min(subrange.end, range.end);
            if value == 1 {
                ranges.remove(subrange);
                let stack = self.stack.lock().unwrap();
                for i in subrange {
                    unsafe {
                        stack[i].assume_init_drop();
                    }
                }
            } else {
                ranges.insert(subrange, value - 1);
            }
        }
    }
}

// {
//     /// Inserts a branch point some number of cells into this cactus's parent.
//     ///
//     /// If there is already a branch at that point, does nothing.
//     fn insert_branch(&mut self, distance: usize) {
//         // If no parent, rebasing will fail to do anything and that's ok. The method
//         // originally called should also fail right after.
//         let Some(parent) = &self.parent else {
//             return;
//         };

//         let mut parent = parent.get();
//         let stack_elements = parent.stack.len();
//         // If the distance exactly equals the number of elements already on this stack, and this
//         // stack already has a parent, then there is already a suitable branch there and no work
//         // needs to be done
//         if stack_elements == distance && parent.parent.is_some() {
//             return;
//         }

//         // If the branch point would land in a grandparent, pass the task up the chain.
//         if stack_elements < distance {
//             parent.insert_branch(distance - stack_elements);
//             return;
//         }

//         // If the branch point would land in the current stack, recreate the parent and then
//         // recreate the self:
//         //     [[], a, b, c, d]
//         // is becoming
//         //     [[[], a, b], c, d]

//         // Remove the branched children from the parent
//         let rest = parent.stack.split_off(stack_elements - distance);
//         let len = parent.len;
//         // Set the current node's parent to a new node with that little bit of stack
//         let mut grandparent = std::mem::replace(
//             &mut *parent,
//             Cactus {
//                 parent: None,
//                 stack: rest,
//                 len,
//             },
//         );
//         grandparent.len = parent.len - parent.stack.len();
//         // And set the parent's parent to the original parent (now grandparent)
//         parent.parent = Some(Parent::new(grandparent));
//     }

//     fn consume_into(mut self, target_stack: &mut Vec<T>, target: usize) -> Option<Parent<T>>
//     where
//         T: Clone,
//     {
//         if self.len() < target {
//             if let Some(parent) = self.parent.take() {
//                 self.parent = parent
//                     .get()
//                     .clone()
//                     .consume_into(target_stack, target - self.stack.len());
//             }
//         }
//         target_stack.append(&mut self.stack);
//         self.parent
//     }

//     /// Consumes parents until the current stack's length is at least the target length.
//     /// Might end up longer depending on the position of the branch points in the parent
//     /// stack, as this will not insert new branches while consuming.
//     ///
//     /// If there are no more parents to consume, then the current stack's resulting length
//     /// will still be less than the requested length.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// # use trilogy_vm::cactus::Cactus;
//     /// let mut cactus = Cactus::new();
//     /// cactus.push(1);
//     /// let mut branch = cactus.branch();
//     /// branch.push(2);
//     /// branch.push(3);
//     /// let mut sub_branch = branch.branch();
//     /// sub_branch.push(4);
//     /// sub_branch.consume_to_length(2);
//     /// assert_eq!(sub_branch.len(), 3);
//     /// assert_eq!(sub_branch.count(), 4);
//     /// ```
//     pub fn consume_to_length(&mut self, target: usize)
//     where
//         T: Clone,
//     {
//         if self.stack.len() >= target {
//             return;
//         }
//         let mut target_stack = Vec::with_capacity(target + self.stack.capacity());
//         if let Some(parent) = self.parent.take() {
//             self.parent = parent
//                 .get()
//                 .clone()
//                 .consume_into(&mut target_stack, target - self.stack.len());
//         }
//         target_stack.append(&mut self.stack);
//         self.stack = target_stack;
//     }

//     /// Consumes parents until the current stack's length is exactly the target length.
//     /// If necessary, this will insert a branch point at the specified distance into the
//     /// stack's parent.
//     ///
//     /// If there are no more parents to consume, then the current stack's resulting length
//     /// will still be less than the requested length.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// # use trilogy_vm::cactus::Cactus;
//     /// let mut cactus = Cactus::new();
//     /// cactus.push(1);
//     /// let mut branch = cactus.branch();
//     /// branch.push(2);
//     /// branch.push(3);
//     /// let mut sub_branch = branch.branch();
//     /// sub_branch.push(4);
//     /// sub_branch.consume_exact(2);
//     /// assert_eq!(sub_branch.len(), 2);
//     /// assert_eq!(sub_branch.count(), 4);
//     /// ```
//     pub fn consume_exact(&mut self, target: usize)
//     where
//         T: Clone,
//     {
//         if target > self.len() {
//             self.insert_branch(target - self.stack.len());
//             self.consume_to_length(target);
//         }
//     }

//     /// Add a value to the end of this cactus.
//     ///
//     /// If the cactus is at capacity, this may trigger a reallocation.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// # use trilogy_vm::cactus::Cactus;
//     /// let mut cactus = Cactus::new();
//     /// cactus.push(1);
//     /// assert_eq!(cactus.at(0), Some(1));
//     /// ```
//     #[inline(always)]
//     pub fn push(&mut self, value: T) {
//         self.len += 1;
//         self.stack.push(value);
//     }

//     /// Pop the topmost value from this cactus.
//     ///
//     /// If the current branch's stack is empty, consumes exactly one value from the parent
//     /// and pops that. This preserves the sharedness of more distant elements.
//     ///
//     /// Popping repeatedly can get inefficient, if you expect to pop more than once, consider
//     /// using [`detach_at`][Self::detach_at] or explicitly preparing more elements to be popped with
//     /// [`consume_exact`][Self::consume_exact].
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// # use trilogy_vm::cactus::Cactus;
//     /// let mut cactus = Cactus::new();
//     /// cactus.push(1);
//     /// cactus.push(2);
//     /// assert_eq!(cactus.pop(), Some(2));
//     /// assert_eq!(cactus.pop(), Some(1));
//     /// ```
//     #[inline(always)]
//     pub fn pop(&mut self) -> Option<T>
//     where
//         T: Clone,
//     {
//         if self.len == 0 {
//             return None;
//         }
//         if self.stack.is_empty() {
//             while self.reduce() {}
//             self.consume_exact(1);
//         }
//         self.len = self.len.saturating_sub(1);
//         self.stack.pop()
//     }

//     #[inline(always)]
//     fn reduce(&mut self) -> bool {
//         let can_take = self
//             .parent
//             .as_ref()
//             .map(|arc| Parent::strong_count(arc) == 1)
//             .unwrap_or(false);
//         if !can_take {
//             return false;
//         }
//         let parent = Parent::into_inner(self.parent.take().unwrap()).unwrap();
//         let mut old_self = std::mem::replace(self, parent);
//         if !old_self.stack.is_empty() {
//             self.stack.append(&mut old_self.stack);
//             self.len = old_self.len;
//         }
//         true
//     }

//     /// Moves all the values in this current branch into a new parent.
//     ///
//     /// The current stack will be empty afterwards.
//     #[inline(always)]
//     fn commit(&mut self) {
//         if !self.stack.is_empty() {
//             let len = self.len;
//             let parent = Parent::new(std::mem::take(self));
//             *self = Self {
//                 parent: Some(parent),
//                 stack: Vec::with_capacity(self.capacity()),
//                 len,
//             };
//         }
//     }

//     /// Branches a cactus into two. Returns one of them, and replaces
//     /// self with the other.
//     ///
//     /// The shared parent of both will be what was the current cactus,
//     /// and both of the new branches will hold 0 active elements.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// # use trilogy_vm::cactus::Cactus;
//     /// let mut cactus = Cactus::new();
//     /// cactus.push(1);
//     /// let mut branch = cactus.branch();
//     /// assert_eq!(cactus.len(), 0);
//     /// assert_eq!(branch.len(), 0);
//     /// assert_eq!(cactus.count(), 1);
//     /// assert_eq!(branch.count(), 1);
//     /// assert_eq!(cactus.pop(), Some(1));
//     /// assert_eq!(branch.pop(), Some(1));
//     /// ```
//     #[inline(always)]
//     pub fn branch(&mut self) -> Self {
//         self.commit();
//         Self {
//             parent: self.parent.clone(),
//             stack: Vec::with_capacity(self.capacity()),
//             len: self.len,
//         }
//     }

//     /// Retrieves a value from the cactus at a specific offset from the top.
//     /// Returns `None` if the offset is out of bounds.
//     ///
//     /// This is an efficient operation for elements in the live portion of
//     /// the current branch, but can be costly to access elements in parent
//     /// stacks.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// # use trilogy_vm::cactus::Cactus;
//     /// let mut cactus = Cactus::new();
//     /// cactus.push(1);
//     /// cactus.push(2);
//     /// assert_eq!(cactus.at(0), Some(2));
//     /// assert_eq!(cactus.at(1), Some(1));
//     /// ```
//     #[inline(always)]
//     pub fn at(&self, offset: usize) -> Option<T>
//     where
//         T: Clone,
//     {
//         let len = self.stack.len();
//         if len > offset {
//             self.stack.get(len - offset - 1).cloned()
//         } else if offset >= self.len {
//             None
//         } else {
//             self.at_impl(offset)
//         }
//     }

//     fn at_impl(&self, offset: usize) -> Option<T>
//     where
//         T: Clone,
//     {
//         let len = self.stack.len();
//         if len > offset {
//             self.stack.get(len - offset - 1).cloned()
//         } else {
//             let mut parent = self.parent.as_ref()?.get();
//             while parent.reduce() {}
//             parent.at_impl(offset - len)
//         }
//     }

//     /// Replace a value in this cactus at a specific distance from the top with a new value.
//     ///
//     /// If the value is in a parent (`offset > self.len()`), the change will be reflected in
//     /// all other branches that share that parent.
//     ///
//     /// # Errors
//     ///
//     /// `OutOfBounds` if the index is in the current stack's live elements, or beyond
//     /// the last parent.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// # use trilogy_vm::cactus::Cactus;
//     /// let mut cactus = Cactus::new();
//     /// cactus.push(1);
//     /// let mut branch = cactus.branch();
//     /// branch.push(2);
//     /// branch.replace_at(1, 3);
//     /// assert_eq!(branch.at(1), Some(3));
//     /// assert_eq!(cactus.at(0), Some(3));
//     /// ```
//     pub fn replace_at(&mut self, offset: usize, value: T) -> Result<T, OutOfBounds> {
//         let len = self.stack.len();
//         if offset < len {
//             Ok(std::mem::replace(
//                 self.stack.get_mut(len - offset - 1).unwrap(),
//                 value,
//             ))
//         } else if let Some(parent) = self.parent.as_ref() {
//             let mut parent = parent.get();
//             parent.replace_at(offset - len, value)
//         } else {
//             Err(OutOfBounds)
//         }
//     }

//     /// Replace a shared value in this cactus with another.
//     ///
//     /// This does not require a mutable reference as the shared portion of the cactus
//     /// is accessed via interior mutability. The tradeoff is that values in the unshared
//     /// portion of this live cactus cannot be replaced.
//     ///
//     /// # Errors
//     ///
//     /// `OutOfBounds` if the index is in the current stack's live elements, or beyond
//     /// the last parent.
//     pub fn replace_shared(&self, offset: usize, value: T) -> Result<T, OutOfBounds> {
//         let len = self.stack.len();
//         if offset < len {
//             Err(OutOfBounds)
//         } else if let Some(parent) = self.parent.as_ref() {
//             let mut parent = parent.get();
//             parent.replace_at(offset - len, value)
//         } else {
//             Err(OutOfBounds)
//         }
//     }

//     /// Detaches in bulk a chunk of values off the top of the stack.
//     ///
//     /// The returned `Vec` of elements has them *in stack order*. That is, the opposite
//     /// order of what they would be if you popped them individually one at a time.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// # use trilogy_vm::cactus::Cactus;
//     /// let mut cactus = Cactus::new();
//     /// cactus.push(1);
//     /// cactus.push(2);
//     /// cactus.push(3);
//     /// assert_eq!(cactus.detach_at(2), vec![2, 3]);
//     /// ```
//     pub fn detach_at(&mut self, count: usize) -> Vec<T>
//     where
//         T: Clone,
//     {
//         if self.stack.len() < count {
//             while self.reduce() {}
//             self.consume_exact(count);
//         }
//         self.len -= count;
//         self.stack.split_off(self.stack.len() - count)
//     }

//     /// Discards a number of elements from this stack.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// # use trilogy_vm::cactus::Cactus;
//     /// let mut cactus = Cactus::new();
//     /// cactus.push(1);
//     /// cactus.push(2);
//     /// let mut branch = cactus.branch();
//     /// branch.push(3);
//     /// branch.discard(2);
//     /// assert_eq!(branch.pop(), Some(1));
//     /// assert_eq!(cactus.pop(), Some(2));
//     /// ```
//     pub fn discard(&mut self, mut count: usize)
//     where
//         T: Clone,
//     {
//         if self.stack.len() >= count {
//             self.len -= count;
//             self.stack.truncate(self.stack.len() - count);
//             return;
//         }

//         count -= self.stack.len();
//         let mut parent = self.parent.take();
//         loop {
//             let Some(par) = parent else {
//                 *self = Cactus::new();
//                 return;
//             };
//             let par = par.get();
//             if par.stack.len() >= count {
//                 *self = par.clone();
//                 break;
//             }
//             count -= par.stack.len();
//             parent = par.parent.clone();
//         }
//         self.len -= count;
//         self.stack.truncate(self.stack.len() - count);
//     }

//     /// Attaches in bulk a chunk of elements to the live branch. These items
//     /// will be pushed in order.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// # use trilogy_vm::cactus::Cactus;
//     /// let mut cactus = Cactus::new();
//     /// cactus.attach(vec![1, 2, 3]);
//     /// assert_eq!(cactus.at(0), Some(3));
//     /// assert_eq!(cactus.at(1), Some(2));
//     /// assert_eq!(cactus.at(2), Some(1));
//     /// ```
//     pub fn attach(&mut self, items: Vec<T>) {
//         self.len += items.len();
//         self.stack.extend(items);
//     }

//     #[doc(hidden)]
//     pub fn iter(&self) -> impl Iterator<Item = T>
//     where
//         T: Clone,
//     {
//         self.clone().into_iter()
//     }
// }

pub struct CactusIntoIter<T>(Cactus<T>);

impl<T: Clone> Iterator for CactusIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl<T: Clone> IntoIterator for Cactus<T> {
    type Item = T;
    type IntoIter = CactusIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        CactusIntoIter(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct OutOfBounds;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cactus_push_pop() {
        let mut cactus = Cactus::new();
        cactus.push(3);
        cactus.push(4);
        cactus.push(5);
        assert_eq!(cactus.pop(), Some(5));
        assert_eq!(cactus.pop(), Some(4));
        assert_eq!(cactus.pop(), Some(3));
        assert_eq!(cactus.pop(), None);
    }

    #[test]
    fn cactus_branch() {
        let mut cactus = Cactus::new();
        cactus.push(3);
        let mut branch = cactus.branch();
        branch.push(4);
        cactus.push(5);
        assert_eq!(cactus.pop(), Some(5));
        assert_eq!(cactus.pop(), Some(3));
        assert_eq!(cactus.pop(), None);
        assert_eq!(branch.pop(), Some(4));
        assert_eq!(branch.pop(), Some(3));
        assert_eq!(branch.pop(), None);
    }

    #[test]
    fn cactus_move_branch_point_pop() {
        let mut cactus = Cactus::new();
        cactus.push(1);
        cactus.push(2);
        let mut branch = cactus.branch();
        assert_eq!(branch.pop(), Some(2));
        branch.replace_at(0, 3).unwrap();
        assert_eq!(branch.pop(), Some(3), "value was set in the new branch");
        assert_eq!(cactus.pop(), Some(2));
        assert_eq!(
            cactus.pop(),
            Some(3),
            "shared value was set in the original too"
        );
    }

    #[test]
    fn cactus_move_branch_point_detach() {
        let mut cactus = Cactus::new();
        cactus.push(1);
        cactus.push(2);
        let mut branch = cactus.branch();
        branch.push(3);
        assert_eq!(branch.detach_at(2), vec![2, 3]);
        branch.replace_at(0, 3).unwrap();
        assert_eq!(branch.pop(), Some(3), "value was set in the new branch");
        assert_eq!(cactus.pop(), Some(2));
        assert_eq!(
            cactus.pop(),
            Some(3),
            "shared value was set in the original too"
        );
    }

    #[test]
    fn cactus_len() {
        let mut cactus = Cactus::new();
        cactus.push(3);
        cactus.push(4);
        cactus.push(5);
        assert_eq!(cactus.count(), 3);
        cactus.pop();
        assert_eq!(cactus.count(), 2);
        let mut branch = cactus.branch();
        branch.push(5);
        assert_eq!(cactus.count(), 2);
        assert_eq!(branch.count(), 3);

        cactus.push(6);
        cactus.detach_at(2);
        assert_eq!(branch.count(), 3);
        assert_eq!(cactus.count(), 1);
    }
}
