//! Implementation of a cactus stack that tries to be a standard stack for as
//! long as possible.
//!
//! Initially very naive, the implementation will hopefully evolve over time
//! into something actually useful, but for now, no sense in going for more
//! than "functioning".

use std::sync::Arc;

#[derive(Clone, Debug)]
pub(crate) struct Cactus<T> {
    parent: Option<Arc<Cactus<T>>>,
    stack: Vec<T>,
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
            *self = (**self.parent.as_ref()?).clone();
        }
        self.stack.pop()
    }

    pub fn peek(&self) -> Option<&T> {
        self.stack.last()
    }

    pub fn branch(&mut self) -> Self {
        let arced = Arc::new(std::mem::take(self));
        *self = Self {
            parent: Some(arced.clone()),
            stack: vec![],
        };
        Self {
            parent: Some(arced),
            stack: vec![],
        }
    }

    pub fn parent(&self) -> Option<Arc<Self>> {
        self.parent.clone()
    }

    pub fn detach(&mut self) -> Option<Arc<Self>> {
        self.parent.take()
    }

    pub fn graft(&mut self, mut child: Cactus<T>) -> Option<Arc<Self>> {
        let parent = Arc::new(std::mem::take(self));
        let prev_parent = child.parent.replace(parent);
        *self = child;
        prev_parent
    }
}

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

    #[test]
    fn cactus_detach() {
        let mut cactus = Cactus::new();
        cactus.push(3);
        let mut branch = cactus.branch();
        branch.push(4);
        let mut parent = (*branch.detach().unwrap()).clone();
        assert_eq!(parent.pop(), Some(3));
        assert_eq!(parent.pop(), None);
        assert_eq!(branch.pop(), Some(4));
        assert_eq!(branch.pop(), None);
    }

    #[test]
    fn cactus_graft() {
        let mut cactus = Cactus::new();
        cactus.push(3);
        let mut branch = cactus.branch();
        branch.push(4);
        cactus.push(5);
        let mut original_parent = (*cactus.graft(branch).unwrap()).clone();
        assert_eq!(cactus.pop(), Some(4));
        assert_eq!(cactus.pop(), Some(5));
        assert_eq!(cactus.pop(), Some(3));
        assert_eq!(cactus.pop(), None);
        assert_eq!(original_parent.pop(), Some(3));
        assert_eq!(original_parent.pop(), None);
    }
}
