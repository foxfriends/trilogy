use super::{Branch, RangeMap};
use std::ops::Range;
use std::sync::{Mutex, MutexGuard, PoisonError};

/// The root of the Cactus Stack.
///
/// The actual stack itself is accessed through `Branch`es.
#[derive(Debug)]
pub struct Cactus<T> {
    /// The backing memory of this stack. This space is sparse.
    stack: Mutex<Vec<Option<T>>>,
    /// Freezes the capacity of the backing vector. Any attempt to push or append
    /// values beyond the capacity will fail.
    capacity_frozen: bool,
}

impl<T> Default for Cactus<T> {
    #[inline]
    fn default() -> Self {
        Self {
            stack: Default::default(),
            capacity_frozen: false,
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
            capacity_frozen: false,
        }
    }

    /// Freezes the capacity of this cactus. When the capacity is frozen, attempts
    /// to append values will fail if the resulting number of elements would overflow
    /// the stack.
    #[inline]
    pub fn freeze_capacity(&mut self) {
        self.capacity_frozen = true;
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
    #[inline]
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
    #[inline]
    pub fn get_ranges(&self, ranges: Vec<Range<usize>>) -> Option<Vec<T>>
    where
        T: Clone,
    {
        let stack = self.stack.lock().unwrap();
        let expected: usize = ranges.iter().map(|rng| rng.len()).sum();
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

    /// Retains only ranges of values from this cactus where the range map is `true`,
    /// leaving the rest of the cells empty. The length and capacity of the
    /// cactus are unaffected.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::{Cactus, RangeMap};
    /// let cactus = Cactus::new();
    /// cactus.append(&mut vec![1, 2, 3, 4, 5, 6]);
    /// let mut keep = RangeMap::default();
    /// keep.insert(0..2, true);
    /// keep.insert(4..6, true);
    /// cactus.retain_ranges(keep);
    /// assert_eq!(cactus.get(1), Some(2));
    /// assert_eq!(cactus.get(2), None);
    /// assert_eq!(cactus.get(4), Some(5));
    /// ```
    #[inline]
    pub fn retain_ranges(&self, ranges: RangeMap<bool>)
    where
        T: Clone,
    {
        let mut stack = self.stack.lock().unwrap();
        for (range, _) in ranges.iter().filter(|(_, v)| !v) {
            for i in range {
                stack[i] = None;
            }
        }
    }

    /// Remove ranges of values from this cactus where the range map is `false`.
    /// Shifts all non-removed ranges towards the front, shortening the length
    /// accordingly. Capacity is not affected.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::{Cactus, RangeMap};
    /// let cactus = Cactus::new();
    /// cactus.append(&mut vec![1, 2, 3, 4, 5, 6]);
    /// let mut keep = RangeMap::default();
    /// keep.insert(0..2, true);
    /// keep.insert(4..6, true);
    /// cactus.remove_ranges(keep);
    /// assert_eq!(cactus.get(1), Some(2));
    /// assert_eq!(cactus.get(2), Some(5));
    /// assert_eq!(cactus.get(4), None);
    /// ```
    #[inline]
    pub fn remove_ranges(&self, ranges: RangeMap<bool>)
    where
        T: Clone,
    {
        let mut stack = self.stack.lock().unwrap();
        for (range, _) in ranges.reverse_iter().filter(|(_, v)| !v) {
            stack.drain(range);
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
    #[inline]
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
    /// # use trilogy_vm::cactus::{Cactus, RangeMap};
    /// let cactus = Cactus::new();
    /// cactus.append(&mut vec![1, 2, 3]);
    /// assert_eq!(cactus.len(), 3);
    /// cactus.retain_ranges(RangeMap::default());
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
    /// # use trilogy_vm::cactus::{Cactus, RangeMap};
    /// let cactus = Cactus::new();
    /// assert!(cactus.is_empty());
    /// cactus.append(&mut vec![1, 2, 3]);
    /// assert!(!cactus.is_empty());
    /// cactus.retain_ranges(RangeMap::default());
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
    /// If the capacity is frozen, may return a `StackOverflow` if the newly appended
    /// elements would cause the underlying vector to reallocate. If capacity is not
    /// frozen, this method will never return `Err`, but it may fail due to the system
    /// allocator failing to allocate memory instead.
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
    pub fn append(&self, values: &mut Vec<T>) -> Result<(), StackOverflow> {
        let mut stack = self.stack.lock().unwrap();
        if self.capacity_frozen && values.len() > stack.capacity() - stack.len() {
            return Err(StackOverflow);
        }
        stack.extend(values.drain(..).map(Some));
        Ok(())
    }

    /// Locks this cactus, returning a handle which can be used to mutate the cactus
    /// atomically.
    ///
    /// Note that attempting to use the regular cactus methods while the cactus
    /// is locked will require the lock to be released, as the cactus is locked
    /// internally to those methods.
    #[inline]
    #[allow(clippy::type_complexity)]
    pub fn lock(&self) -> Result<CactusGuard<T>, PoisonError<MutexGuard<Vec<Option<T>>>>> {
        self.stack.lock().map(CactusGuard)
    }
}

pub struct CactusGuard<'a, T>(MutexGuard<'a, Vec<Option<T>>>);

impl<'a, T> CactusGuard<'a, T> {
    /// Remove ranges of values from this cactus where the range map is `false`.
    /// Shifts all non-removed ranges towards the front, shortening the length
    /// accordingly. Capacity is not affected.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::{Cactus, RangeMap};
    /// let cactus = Cactus::new();
    /// cactus.append(&mut vec![1, 2, 3, 4, 5, 6]);
    /// let mut keep = RangeMap::default();
    /// keep.insert(0..2, true);
    /// keep.insert(4..6, true);
    /// cactus.remove_ranges(keep);
    /// assert_eq!(cactus.get(1), Some(2));
    /// assert_eq!(cactus.get(2), Some(5));
    /// assert_eq!(cactus.get(4), None);
    /// ```
    #[inline]
    pub fn remove_ranges(&mut self, ranges: RangeMap<bool>)
    where
        T: Clone,
    {
        for (range, _) in ranges.reverse_iter().filter(|(_, v)| !v) {
            self.0.drain(range);
        }
    }
}

/// An error that indicates the stack has overflown.
pub struct StackOverflow;
