//! A cactus stack.
//!
//! This is the stack implementation that backs the Trilogy VM, where branches
//! are used to represent continuations and closures that share a parent stack
//! but have differing active stacks.
use std::fmt::{self, Debug};
use std::sync::{Arc, Mutex};

/// A Cactus Stack.
#[derive(Clone)]
pub(crate) struct Cactus<T> {
    parent: Option<Arc<Mutex<Cactus<T>>>>,
    stack: Vec<T>,
    len: usize,
}

impl<T: Debug> Debug for Cactus<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Cactus");
        if let Some(parent) = &self.parent {
            let parent = parent.lock().unwrap();
            debug.field("parent", &Some(&*parent));
        }
        debug.field("stack", &self.stack).finish()
    }
}

impl<T> Default for Cactus<T> {
    fn default() -> Self {
        Self {
            parent: None,
            stack: vec![],
            len: 0,
        }
    }
}

impl<T> Cactus<T> {
    #[cfg(test)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a branch point some number of cells into this cactus's parent.
    fn insert_branch(&mut self, distance: usize) {
        // If no parent, rebasing will fail to do anything and that's ok. The method
        // originally called should also fail right after.
        let Some(parent) = &self.parent else {
            return;
        };

        let mut parent = parent.lock().unwrap();
        let stack_elements = parent.stack.len();
        // If the distance exactly equals the number of elements already on this stack, and this
        // stack already has a parent, then there is already a suitable branch there and no work
        // needs to be done
        if stack_elements == distance && parent.parent.is_some() {
            return;
        }

        // If the branch point would land in a grandparent, pass the task up the chain.
        if stack_elements < distance {
            parent.insert_branch(distance - stack_elements);
            std::mem::drop(parent);
            return;
        }

        // If the branch point would land in the current stack, recreate the parent and then
        // recreate the self:
        //     [[], a, b, c, d]
        // is becoming
        //     [[[], a, b], c, d]

        // Remove the branched children from the parent
        let rest = parent.stack.split_off(stack_elements - distance);
        let len = parent.len;
        // Set the current node's parent to a new node with that little bit of stack
        let mut grandparent = std::mem::replace(
            &mut *parent,
            Cactus {
                parent: None,
                stack: rest,
                len,
            },
        );
        grandparent.len = parent.len - parent.stack.len();
        // And set the parent's parent to the original parent (now grandparent)
        parent.parent = Some(Arc::new(Mutex::new(grandparent)));
    }

    /// Consumes parents until the current stack's length is at least the target length.
    /// Might end up longer depending on the position of the branch points in the parent
    /// stack.
    fn consume_to_length(&mut self, target: usize)
    where
        T: Clone,
    {
        while self.stack.len() < target {
            let Some(parent) = &self.parent.take() else {
                return;
            };
            let Cactus { parent, stack, .. } = parent.lock().unwrap().clone();
            self.parent = parent;
            let mut rest = std::mem::replace(&mut self.stack, stack);
            self.stack.append(&mut rest);
        }
    }

    pub fn push(&mut self, value: T) {
        self.len += 1;
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Option<T>
    where
        T: Clone,
    {
        if self.stack.is_empty() {
            self.insert_branch(1);
            self.consume_to_length(1);
        }
        self.len = self.len.saturating_sub(1);
        self.stack.pop()
    }

    pub fn commit(&mut self) {
        if !self.stack.is_empty() {
            let len = self.len;
            let arced = Arc::new(Mutex::new(std::mem::take(self)));
            *self = Self {
                parent: Some(arced),
                stack: vec![],
                len,
            };
        }
    }

    pub fn branch(&mut self) -> Self {
        self.commit();
        Self {
            parent: self.parent.clone(),
            stack: vec![],
            len: self.len,
        }
    }

    pub fn at(&self, offset: usize) -> Option<T>
    where
        T: Clone,
    {
        let len = self.stack.len();
        if len > offset {
            self.stack.get(len - offset - 1).cloned()
        } else {
            self.parent.as_ref()?.lock().unwrap().at(offset - len)
        }
    }

    pub fn replace_at(&mut self, offset: usize, value: T) -> Result<T, OutOfBounds> {
        let len = self.stack.len();
        if offset < len {
            Ok(std::mem::replace(
                self.stack.get_mut(len - offset - 1).unwrap(),
                value,
            ))
        } else if let Some(parent) = self.parent.as_ref() {
            let mut parent = parent.lock().unwrap();
            parent.replace_at(offset - len, value)
        } else {
            Err(OutOfBounds)
        }
    }

    pub fn detach_at(&mut self, count: usize) -> Option<Vec<T>>
    where
        T: Clone + std::fmt::Debug,
    {
        if self.stack.len() < count {
            self.insert_branch(count - self.stack.len());
            self.consume_to_length(count);
        }
        self.len = self.len.saturating_sub(count);
        Some(self.stack.split_off(self.stack.len() - count))
    }

    pub fn attach(&mut self, items: Vec<T>) {
        self.len += items.len();
        self.stack.extend(items);
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

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
        assert_eq!(branch.detach_at(2), Some(vec![2, 3]));
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
        assert_eq!(cactus.len(), 3);
        cactus.pop();
        assert_eq!(cactus.len(), 2);
        let mut branch = cactus.branch();
        branch.push(5);
        assert_eq!(cactus.len(), 2);
        assert_eq!(branch.len(), 3);

        cactus.push(6);
        cactus.detach_at(2);
        assert_eq!(branch.len(), 3);
        assert_eq!(cactus.len(), 1);
    }
}
