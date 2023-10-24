//! Implementation of a cactus stack that tries to be a standard stack for as
//! long as possible.
//!
//! Initially very naive, the implementation will hopefully evolve over time
//! into something actually useful, but for now, no sense in going for more
//! than "functioning".

use std::fmt::{self, Debug};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(crate) struct Cactus<T> {
    parent: Option<Arc<Mutex<Cactus<T>>>>,
    stack: Vec<T>,
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

impl<T> Eq for Cactus<T> where T: PartialEq {}

impl<T> PartialEq for Cactus<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.stack != other.stack {
            return false;
        }
        match (&self.parent, &other.parent) {
            (Some(lhs), Some(rhs)) => Arc::ptr_eq(lhs, rhs),
            _ => false,
        }
    }
}

impl<T> Default for Cactus<T> {
    fn default() -> Self {
        Self {
            parent: None,
            stack: vec![],
        }
    }
}

impl<T> Cactus<T> {
    #[cfg(test)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, value: T) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Option<T>
    where
        T: Clone,
    {
        while self.stack.is_empty() {
            // TODO: https://doc.rust-lang.org/std/sync/struct.Arc.html#method.unwrap_or_clone
            let new_self = self.parent.as_ref()?.lock().unwrap().clone();
            *self = new_self;
        }
        self.stack.pop()
    }

    #[cfg(test)]
    pub fn peek(&self) -> Option<&T> {
        self.stack.last()
    }

    pub fn commit(&mut self) {
        let arced = Arc::new(Mutex::new(std::mem::take(self)));
        *self = Self {
            parent: Some(arced),
            stack: vec![],
        };
    }

    pub fn branch(&mut self) -> Self {
        self.commit();
        Self {
            parent: self.parent.clone(),
            stack: vec![],
        }
    }

    #[cfg(test)]
    pub fn parent(&self) -> Option<Arc<Mutex<Self>>> {
        self.parent.clone()
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
        while self.stack.len() < count {
            let new_self = self.parent.as_ref()?.lock().unwrap().clone();
            self.parent = new_self.parent;
            let end = std::mem::replace(&mut self.stack, new_self.stack);
            self.stack.extend(end);
        }
        Some(self.stack.split_off(self.stack.len() - count))
    }

    pub fn attach(&mut self, items: Vec<T>) {
        self.stack.extend(items);
    }

    pub fn len(&self) -> usize {
        self.stack.len()
            + self
                .parent
                .as_ref()
                .map(|parent| parent.lock().unwrap().len())
                .unwrap_or(0)
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
    fn cactus_peek() {
        let mut cactus = Cactus::new();
        cactus.push(3);
        assert_eq!(cactus.peek(), Some(&3));
        assert_eq!(cactus.peek(), Some(&3));
        assert_eq!(cactus.pop(), Some(3));
        assert_eq!(cactus.peek(), None);
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
    fn cactus_parent_shared() {
        let mut cactus = Cactus::<()>::new();
        let branch = cactus.branch();
        assert!(Arc::ptr_eq(
            &cactus.parent().unwrap(),
            &branch.parent().unwrap()
        ));
    }
}