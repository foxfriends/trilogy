use super::Branch;
use std::{ops::Range, sync::Mutex};

/// The root of the Cactus Stack.
///
/// The actual stack itself is accessed through `Branch`es.
#[derive(Debug)]
pub struct Cactus<T> {
    /// The backing memory of this stack. This space is sparse.
    stack: Mutex<Vec<Option<T>>>,
}

impl<T> Default for Cactus<T> {
    #[inline]
    fn default() -> Self {
        Self {
            stack: Default::default(),
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
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            stack: Mutex::new(Vec::with_capacity(cap)),
        }
    }

    pub fn get(&self, index: usize) -> Option<T>
    where
        T: Clone,
    {
        self.stack
            .lock()
            .unwrap()
            .get(index)
            .and_then(|v| v.clone())
    }

    pub fn get_ranges(&self, ranges: Vec<Range<usize>>) -> Option<Vec<T>>
    where
        T: Clone,
    {
        let stack = self.stack.lock().unwrap();
        let expected = ranges.iter().map(|rng| rng.len()).sum();
        let vec = ranges
            .into_iter()
            .flat_map(|range| stack.get(range))
            .flatten()
            .cloned()
            .collect::<Option<Vec<_>>>()?;
        if vec.len() != expected {
            None
        } else {
            Some(vec)
        }
    }

    pub fn set(&self, index: usize, value: T) {
        self.stack.lock().unwrap().insert(index, Some(value));
    }

    /// Returns the total number of elements this Cactus can hold without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.stack.lock().unwrap().capacity()
    }

    /// Reserves capacity for at least `additional` more elements to be added to this Cactus.
    #[inline]
    pub fn reserve(&self, additional: usize) {
        self.stack.lock().unwrap().reserve(additional);
    }

    /// Returns a new branch on this cactus. This branch contains no elements (i.e. comes
    /// straight out from the ground).
    #[inline]
    pub fn branch(&self) -> Branch<T> {
        Branch::new(self)
    }

    #[inline]
    pub(super) fn append(&self, values: &mut Vec<T>) -> Range<usize> {
        let mut stack = self.stack.lock().unwrap();
        let range = stack.len()..stack.len() + values.len();
        stack.extend(values.drain(..).map(Some));
        range
    }
}
