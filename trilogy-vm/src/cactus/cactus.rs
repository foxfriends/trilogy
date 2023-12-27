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

    /// Gets a single value from this cactus. Returns `None` if the index
    /// is out of range or has already been deallocated.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::Cactus;
    /// let cactus = Cactus::new();
    /// cactus.append(&mut vec![1, 2, 3]);
    /// assert_eq!(cactus.get(0), Some(1));
    /// assert_eq!(cactus.get(2), Some(3));
    /// assert_eq!(cactus.get(3), None);
    /// ```
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

    /// Gets multiple values from this cactus. Returns `None` if the ranges are not
    /// all completely allocated.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::Cactus;
    /// let cactus = Cactus::new();
    /// cactus.append(&mut vec![1, 2, 3, 4, 5, 6]);
    /// assert_eq!(cactus.get_ranges(vec![0..2, 4..6]), Some(vec![1, 2, 5, 6]));
    /// assert_eq!(cactus.get_ranges(vec![0..2, 4..7]), None);
    /// ```
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

    /// Drop a range of values from this cactus.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::Cactus;
    /// let cactus = Cactus::new();
    /// cactus.append(&mut vec![1, 2, 3, 4, 5, 6]);
    /// cactus.drop_ranges(vec![(2..4)].into_iter());
    /// assert_eq!(cactus.get(1), Some(2));
    /// assert_eq!(cactus.get(2), None);
    /// assert_eq!(cactus.get(4), Some(5));
    /// ```
    pub fn drop_ranges(&self, ranges: impl Iterator<Item = Range<usize>>)
    where
        T: Clone,
    {
        let mut stack = self.stack.lock().unwrap();
        for range in ranges {
            for i in range {
                stack[i] = None;
            }
        }
    }

    /// Sets the value at a specific index in this cactus. Other elements
    /// are unaffected.
    ///
    /// # Panics
    ///
    /// If the index is out of range.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::Cactus;
    /// let cactus = Cactus::new();
    /// cactus.append(&mut vec![1, 2, 3]);
    /// cactus.set(1, 5);
    /// assert_eq!(cactus.get(0), Some(1));
    /// assert_eq!(cactus.get(1), Some(5));
    /// assert_eq!(cactus.get(2), Some(3));
    /// ```
    pub fn set(&self, index: usize, value: T) {
        self.stack.lock().unwrap()[index] = Some(value);
    }

    /// Returns the total number of elements this Cactus can hold without reallocating.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::Cactus;
    /// let cactus = Cactus::with_capacity(5);
    /// cactus.append(&mut vec![1, 2, 3]);
    /// assert!(cactus.capacity() >= 5);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.stack.lock().unwrap().capacity()
    }

    /// Returns the total number of cells in this cactus. Since the cactus may sometimes
    /// be sparse, the length is not the number of values, and some values within the
    /// range of 0 to `len()` may be `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::Cactus;
    /// let cactus = Cactus::new();
    /// cactus.append(&mut vec![1, 2, 3]);
    /// assert_eq!(cactus.len(), 3);
    /// cactus.drop_ranges(vec![(0..3)].into_iter());
    /// assert_eq!(cactus.len(), 3);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.stack.lock().unwrap().len()
    }

    /// Returns true if the cactus has no cells allocated. A non-empty cactus may
    /// still hold no values.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::Cactus;
    /// let cactus = Cactus::new();
    /// assert!(cactus.is_empty());
    /// cactus.append(&mut vec![1, 2, 3]);
    /// assert!(!cactus.is_empty());
    /// cactus.drop_ranges(vec![(0..3)].into_iter());
    /// assert!(!cactus.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.stack.lock().unwrap().is_empty()
    }

    /// Reserves capacity for at least `additional` more elements to be added to this Cactus.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::Cactus;
    /// let cactus = Cactus::new();
    /// cactus.append(&mut vec![1, 2, 3]);
    /// cactus.reserve(5);
    /// assert!(cactus.capacity() >= 8);
    /// ```
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

    /// Appends values to this Cactus. The elements are added to the end of the cactus,
    /// without reusing any of the internal spaces as a result of sparsity.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::Cactus;
    /// let cactus = Cactus::new();
    /// cactus.append(&mut vec![1, 2]);
    /// cactus.append(&mut vec![3, 4]);
    /// assert_eq!(cactus.get(3), Some(4));
    /// ```
    #[inline]
    pub fn append(&self, values: &mut Vec<T>) {
        let mut stack = self.stack.lock().unwrap();
        stack.extend(values.drain(..).map(Some));
    }
}
