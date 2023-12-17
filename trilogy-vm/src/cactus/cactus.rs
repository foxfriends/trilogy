use rangemap::RangeMap;
use std::mem::MaybeUninit;
use std::ops::{DerefMut, Range};
use std::sync::{Arc, Mutex, MutexGuard};

use super::Branch;

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
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            stack: Mutex::new(Vec::with_capacity(cap)),
            ranges: Default::default(),
        }
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

    pub(super) unsafe fn get_release(&self, index: usize) -> T
    where
        T: Clone,
    {
        let ranges = self.ranges.lock().unwrap();
        let stack = self.stack.lock().unwrap();
        let value = stack[index].assume_init_ref().clone();
        self.release_range_from(Some(ranges), Some(stack), index..index + 1);
        value
    }

    pub(super) unsafe fn get_release_ranges(&self, read_ranges: &[Range<usize>]) -> Vec<T>
    where
        T: Clone,
    {
        let len = read_ranges.iter().map(|rng| rng.len()).sum();
        let mut buf = Vec::with_capacity(len);

        let ranges = self.ranges.lock().unwrap();
        let stack = self.stack.lock().unwrap();
        for range in read_ranges {
            buf.extend(
                stack[*range]
                    .iter()
                    .map(|val| val.assume_init_ref().clone()),
            );
            self.release_range_from(Some(ranges), Some(stack), *range)
        }
        buf
    }

    pub(super) unsafe fn get_unchecked(&self, index: usize) -> Option<T>
    where
        T: Clone,
    {
        self.stack
            .lock()
            .unwrap()
            .get(index)
            .map(|val| val.assume_init_ref().clone())
    }

    pub(super) unsafe fn set_unchecked(&self, index: usize, value: T) {
        self.stack.lock().unwrap()[index].write(value);
    }

    #[inline]
    pub(super) fn acquire_ranges(&self, ranges_to_acquire: &[Range<usize>]) {
        let mut ranges = self.ranges.lock().unwrap();
        for range in ranges_to_acquire {
            self.acquire_range_from(Some(ranges), *range)
        }
    }

    #[inline]
    pub(super) fn acquire_range(&self, range: Range<usize>) {
        self.acquire_range_from(None, range)
    }

    #[inline]
    pub(super) fn release(&self, index: usize) {
        self.release_range(index..index + 1);
    }

    #[inline]
    pub(super) fn release_range(&self, range: Range<usize>) {
        self.release_range_from(None, None, range)
    }

    #[inline]
    pub(super) fn acquire_range_from(
        &self,
        ranges: Option<MutexGuard<RangeMap<usize, usize>>>,
        range: Range<usize>,
    ) {
        let mut ranges = ranges.unwrap_or_else(|| self.ranges.lock().unwrap());
        for (&subrange, &value) in ranges.overlapping(&range) {
            let subrange =
                usize::max(subrange.start, range.start)..usize::min(subrange.end, range.end);
            ranges.insert(subrange, value + 1);
        }
    }

    #[inline]
    fn release_range_from(
        &self,
        ranges: Option<MutexGuard<RangeMap<usize, usize>>>,
        mut stack: Option<MutexGuard<Vec<MaybeUninit<T>>>>,
        range: Range<usize>,
    ) {
        let mut ranges = ranges.unwrap_or_else(|| self.ranges.lock().unwrap());
        for (&subrange, &value) in ranges.overlapping(&range) {
            let subrange =
                usize::max(subrange.start, range.start)..usize::min(subrange.end, range.end);
            if value == 1 {
                ranges.remove(subrange);
                let stack = stack.get_or_insert_with(|| self.stack.lock().unwrap());
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

    #[inline]
    pub(super) fn append(&self, values: &mut Vec<T>) -> Range<usize> {
        let ranges = self.ranges.lock().unwrap();
        let mut stack = self.stack.lock().unwrap();
        let len = values.len();
        let range = stack.len()..stack.len() + len;
        stack.extend(values.drain(..).map(MaybeUninit::new));
        self.acquire_range_from(Some(ranges), range);
        range
    }
}
