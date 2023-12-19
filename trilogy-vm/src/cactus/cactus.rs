use rangemap::RangeMap;
use std::fmt::Debug;
use std::mem::MaybeUninit;
use std::ops::Range;
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

impl<T: Debug> Debug for Cactus<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ranges = self.ranges.lock().unwrap();
        let stack = self.stack.lock().unwrap();
        let elements = ranges
            .iter()
            .flat_map(|(rng, _)| rng.clone())
            .map(|i| unsafe { stack[i].assume_init_ref() })
            .collect::<Vec<_>>();
        f.debug_struct("Cactus")
            .field("ranges", &*ranges)
            .field("stack", &elements)
            .finish()
    }
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

impl<T> Drop for Cactus<T> {
    fn drop(&mut self) {
        let ranges = self.ranges.lock().unwrap();
        let mut stack = self.stack.lock().unwrap();
        for (range, value) in ranges.iter() {
            if *value > 0 {
                for val in &mut stack[range.clone()] {
                    unsafe {
                        val.assume_init_drop();
                    }
                }
            }
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

    #[inline]
    pub(super) unsafe fn get_release(&self, index: usize) -> T
    where
        T: Clone,
    {
        let mut ranges = self.ranges.lock().unwrap();
        let mut stack = self.stack.lock().unwrap();
        let value = stack[index].assume_init_ref().clone();
        self.release_range_from(&mut ranges, &mut stack, index..index + 1);
        value
    }

    #[inline]
    pub(super) unsafe fn get_release_ranges(&self, read_ranges: &[Range<usize>]) -> Vec<T>
    where
        T: Clone,
    {
        let len = read_ranges.iter().map(|rng| rng.len()).sum();
        let mut buf = Vec::with_capacity(len);

        let mut ranges = self.ranges.lock().unwrap();
        let mut stack = self.stack.lock().unwrap();
        for range in read_ranges {
            buf.extend(
                stack[range.clone()]
                    .iter()
                    .map(|val| val.assume_init_ref().clone()),
            );
            self.release_range_from(&mut ranges, &mut stack, range.clone())
        }
        buf
    }

    #[inline]
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

    #[inline]
    pub(super) unsafe fn set_unchecked(&self, index: usize, value: T) {
        self.stack.lock().unwrap()[index].write(value);
    }

    #[inline]
    pub(super) fn acquire_ranges(&self, ranges_to_acquire: &[Range<usize>]) {
        let mut ranges = self.ranges.lock().unwrap();
        for range in ranges_to_acquire {
            self.acquire_range_from(&mut ranges, range.clone())
        }
    }

    #[inline]
    pub(super) fn acquire_range(&self, range: Range<usize>) {
        let mut ranges = self.ranges.lock().unwrap();
        self.acquire_range_from(&mut ranges, range)
    }

    #[inline]
    pub(super) fn release(&self, index: usize) {
        self.release_range(index..index + 1);
    }

    #[inline]
    pub(super) fn release_range(&self, range: Range<usize>) {
        let mut ranges = self.ranges.lock().unwrap();
        let mut stack = self.stack.lock().unwrap();
        self.release_range_from(&mut ranges, &mut stack, range)
    }

    #[inline]
    pub(super) fn release_ranges(&self, ranges_to_release: &[Range<usize>]) {
        let mut ranges = self.ranges.lock().unwrap();
        let mut stack = self.stack.lock().unwrap();
        for range in ranges_to_release {
            self.release_range_from(&mut ranges, &mut stack, range.clone())
        }
    }

    #[inline]
    pub(super) fn acquire_range_from(
        &self,
        ranges: &mut MutexGuard<RangeMap<usize, usize>>,
        range: Range<usize>,
    ) {
        let ranges_acquired = ranges
            .overlapping(&range)
            .map(|(subrange, value)| {
                let subrange =
                    usize::max(subrange.start, range.start)..usize::min(subrange.end, range.end);
                (subrange, value + 1)
            })
            .chain(ranges.gaps(&range).map(|gap| (gap, 1)))
            .collect::<Vec<_>>();
        for (range, value) in ranges_acquired {
            ranges.insert(range, value);
        }
    }

    #[inline]
    fn release_range_from(
        &self,
        ranges: &mut MutexGuard<RangeMap<usize, usize>>,
        stack: &mut MutexGuard<Vec<MaybeUninit<T>>>,
        range: Range<usize>,
    ) {
        let ranges_to_remove = ranges
            .overlapping(&range)
            .map(|(subrange, value)| {
                let subrange =
                    usize::max(subrange.start, range.start)..usize::min(subrange.end, range.end);
                (subrange, value - 1)
            })
            .collect::<Vec<_>>();
        for (range, value) in ranges_to_remove {
            if value == 0 {
                ranges.remove(range.clone());
                for i in range {
                    unsafe {
                        stack[i].assume_init_drop();
                    }
                }
            } else {
                ranges.insert(range, value);
            }
        }
    }

    #[inline]
    pub(super) fn append(&self, values: &mut Vec<T>) -> Range<usize> {
        let mut ranges = self.ranges.lock().unwrap();
        let mut stack = self.stack.lock().unwrap();
        let len = values.len();
        let range = stack.len()..stack.len() + len;
        stack.extend(values.drain(..).map(MaybeUninit::new));
        self.acquire_range_from(&mut ranges, range.clone());
        range
    }
}
