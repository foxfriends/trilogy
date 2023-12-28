//! A range map implementation specialized for usize ranges of usize values.
//!
//! Originally this used the `rangemap` crate, but that proved to be significantly
//! too slow. This implementation though less general is a lot faster for our purposes.
use std::collections::BTreeMap;
use std::ops::{Bound, Range};

mod pairwise;
use pairwise::*;

/// A map of ranges to values.
///
/// This map is specialized for `usize` ranges to `Copy` values.
#[derive(Clone, Debug)]
pub struct RangeMap<T>(BTreeMap<usize, T>);

impl<T: Default> Default for RangeMap<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> RangeMap<T> {
    /// Creates a new empty `RangeMap` with a given initial value for all elements.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::RangeMap;
    /// let map = RangeMap::new(0);
    /// ```
    pub fn new(init: T) -> Self {
        let mut map = BTreeMap::new();
        map.insert(0, init);
        RangeMap(map)
    }

    /// Returns the "length" of this RangeMap. The length is the end of the last range for
    /// which a value is set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::RangeMap;
    /// let mut map = RangeMap::new(0);
    /// assert_eq!(map.len(), 0);
    /// map.insert(3..5, 3);
    /// assert_eq!(map.len(), 5);
    /// ```
    pub fn len(&self) -> usize {
        *self.0.last_key_value().unwrap().0
    }

    /// Returns true if this RangeMap is "empty".
    ///
    /// Since a RangeMap always contains at least one range (of the entire domain), the
    /// RangeMap is considered empty if it contains only that range.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::RangeMap;
    /// let mut map = RangeMap::new(0);
    /// assert!(map.is_empty());
    /// map.insert(3..5, 3);
    /// assert!(!map.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.len() == 1
    }

    /// An iterator over all contiguous ranges to the same value in this RangeMap.
    ///
    /// This iterator does not include the infinite range at the end of the map.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::RangeMap;
    /// let mut map = RangeMap::default();
    /// map.insert(2..4, 1);
    /// map.insert(6..8, 2);
    /// let ranges = map.iter().collect::<Vec<_>>();
    /// assert_eq!(ranges, vec![(0..2, 0), (2..4, 1), (4..6, 0), (6..8, 2)]);
    /// ```
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (Range<usize>, T)> + ExactSizeIterator + '_
    where
        T: Copy,
    {
        self.0
            .iter()
            .peekable()
            .pairwise()
            .map(|((s, v), (e, _))| (*s..*e, *v))
    }

    /// An iterator over all contiguous ranges to the same value in this RangeMap, starting from the end.
    ///
    /// This iterator does not include the infinite range at the end of the map.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::RangeMap;
    /// let mut map = RangeMap::default();
    /// map.insert(2..4, 1);
    /// map.insert(6..8, 2);
    /// let ranges = map.reverse_iter().collect::<Vec<_>>();
    /// assert_eq!(ranges, vec![(6..8, 2), (4..6, 0), (2..4, 1), (0..2, 0)]);
    /// ```
    #[inline]
    pub fn reverse_iter(&self) -> impl Iterator<Item = (Range<usize>, T)> + ExactSizeIterator + '_
    where
        T: Copy,
    {
        self.0
            .iter()
            .rev()
            .peekable()
            .pairwise()
            .map(|((e, _), (s, v))| (*s..*e, *v))
    }

    /// An iterator over the overlapping ranges and values of the given range. The
    /// ranges emitted in this iterator will span the entire origin range.
    ///
    /// This includes ranges with a value of 0.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::RangeMap;
    /// let mut map = RangeMap::default();
    /// map.insert(2..4, 1);
    /// map.insert(6..8, 2);
    /// let ranges = map.range(1..9).collect::<Vec<_>>();
    /// assert_eq!(ranges, vec![
    ///     ((1..2), 0),
    ///     ((2..4), 1),
    ///     ((4..6), 0),
    ///     ((6..8), 2),
    ///     ((8..9), 0),
    /// ]);
    ///
    /// let ranges = map.range(2..4).collect::<Vec<_>>();
    /// assert_eq!(ranges, vec![
    ///     ((2..4), 1),
    /// ]);
    /// ```
    #[inline]
    pub fn range(&self, range: Range<usize>) -> impl Iterator<Item = (Range<usize>, T)> + '_
    where
        T: Copy + Default,
    {
        if range.is_empty() {
            return Box::new(std::iter::empty()) as Box<dyn Iterator<Item = (Range<usize>, T)>>;
        }
        let start_val = self.get(range.start);
        Box::new(
            std::iter::once((range.start, *start_val))
                .chain(
                    self.0
                        .range((Bound::Excluded(range.start), Bound::Excluded(range.end)))
                        .map(|(s, v)| (*s, *v)),
                )
                .chain(std::iter::once((range.end, T::default())))
                .peekable()
                .pairwise()
                .map(|((s, v), (e, _))| (s..e, v)),
        )
    }

    /// Gets the value at a specific index. If the index is not included in any
    /// range, then its value is 0.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::RangeMap;
    /// let mut map = RangeMap::default();
    /// map.insert(2..4, 1);
    /// map.insert(6..8, 2);
    /// assert_eq!(map.get(0), &0);
    /// assert_eq!(map.get(2), &1);
    /// assert_eq!(map.get(7), &2);
    /// assert_eq!(map.get(8), &0);
    /// ```
    #[inline]
    pub fn get(&self, key: usize) -> &T {
        self.0
            .range((Bound::Unbounded, Bound::Included(key)))
            .last()
            .unwrap()
            .1
    }

    /// Gets the value before a specific index. Returns `None` if the index is
    /// 0, as there is nothing before 0.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::RangeMap;
    /// let mut map = RangeMap::default();
    /// map.insert(2..4, 1);
    /// map.insert(6..8, 2);
    /// assert_eq!(map.before(0), None);
    /// assert_eq!(map.before(2), Some(&0));
    /// assert_eq!(map.before(7), Some(&2));
    /// assert_eq!(map.before(8), Some(&2));
    /// assert_eq!(map.before(9), Some(&0));
    /// ```
    #[inline]
    pub fn before(&self, key: usize) -> Option<&T> {
        self.0
            .range((Bound::Unbounded, Bound::Excluded(key)))
            .last()
            .map(|kv| kv.1)
    }

    /// Inserts a range into this map. If the range overlaps ranges already included
    /// in the map, the overlapping portions will be overwritten.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::RangeMap;
    /// let mut map = RangeMap::default();
    /// map.insert(2..8, 1);
    /// map.insert(6..7, 2);
    /// map.insert(1..4, 3);
    /// assert_eq!(map.get(0), &0);
    /// assert_eq!(map.get(1), &3);
    /// assert_eq!(map.get(2), &3);
    /// assert_eq!(map.get(3), &3);
    /// assert_eq!(map.get(4), &1);
    /// assert_eq!(map.get(5), &1);
    /// assert_eq!(map.get(6), &2);
    /// assert_eq!(map.get(7), &1);
    /// assert_eq!(map.get(8), &0);
    /// ```
    #[inline]
    pub fn insert(&mut self, range: Range<usize>, value: T)
    where
        T: Eq + Copy,
    {
        if range.is_empty() {
            return;
        }
        let before = self.before(range.start).copied();
        let after = *self.get(range.end);
        let keys_to_remove = self
            .0
            .range((Bound::Excluded(range.start), Bound::Excluded(range.end)))
            .map(|(k, _)| *k)
            .collect::<Vec<_>>();
        for key in keys_to_remove {
            self.0.remove(&key);
        }
        if after == value {
            self.0.remove(&range.end);
        } else {
            self.0.insert(range.end, after);
        }
        if before == Some(value) {
            self.0.remove(&range.start);
        } else {
            self.0.insert(range.start, value);
        }
    }

    /// Inserts the unbounded range `from..` to be the given value. All ranges
    /// after the `from` index are removed, and replaced with the single tail range.
    /// As usual, the tail range is not reflected in the iterator over ranges, but is
    /// used when large keys are requested.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::RangeMap;
    /// let mut map = RangeMap::default();
    /// map.insert(2..8, 1);
    /// map.insert_tail(6, 5);
    /// assert_eq!(map.get(4), &1);
    /// assert_eq!(map.get(6), &5);
    /// assert_eq!(map.get(8), &5);
    /// assert_eq!(map.get(6000), &5);
    /// ```
    pub fn insert_tail(&mut self, from: usize, value: T)
    where
        T: Eq,
    {
        let tail_keys = self
            .0
            .range((Bound::Excluded(from), Bound::Unbounded))
            .map(|(k, _)| *k)
            .collect::<Vec<_>>();
        for key in tail_keys {
            self.0.remove(&key);
        }
        if self.get(from) != &value {
            self.0.insert(from, value);
        }
    }

    /// Removes a range into this map. If the range overlaps ranges already included
    /// in the map, the overlapping portions will be overwritten.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::RangeMap;
    /// let mut map = RangeMap::default();
    /// map.insert(2..6, 1);
    /// map.remove(1..4);
    /// assert_eq!(map.get(0), &0);
    /// assert_eq!(map.get(1), &0);
    /// assert_eq!(map.get(2), &0);
    /// assert_eq!(map.get(3), &0);
    /// assert_eq!(map.get(4), &1);
    /// assert_eq!(map.get(5), &1);
    /// assert_eq!(map.get(6), &0);
    /// ```
    #[inline]
    pub fn remove(&mut self, range: Range<usize>)
    where
        T: Default + Eq + Copy,
    {
        if range.is_empty() {
            return;
        }
        self.insert(range, T::default());
    }

    /// Updates existing ranges in this map in-place by calling the transformation
    /// function. Only the overlapping portions of ranges that partially overlap
    /// the given range will be updated. Ranges where the value is 0 are passed
    /// to the update function as well, providing the opportunity to initialize them
    /// if necessary.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::RangeMap;
    /// let mut map = RangeMap::default();
    /// map.insert(2..4, 1);
    /// map.insert(4..6, 2);
    /// map.update(0..7, |x| { *x = (*x + 1) * 2; });
    /// assert_eq!(map.get(0), &2);
    /// assert_eq!(map.get(1), &2);
    /// assert_eq!(map.get(2), &4);
    /// assert_eq!(map.get(3), &4);
    /// assert_eq!(map.get(4), &6);
    /// assert_eq!(map.get(5), &6);
    /// assert_eq!(map.get(6), &2);
    /// assert_eq!(map.get(7), &0);
    /// ```
    #[inline]
    pub fn update<F: Fn(&mut T)>(&mut self, range: Range<usize>, f: F)
    where
        T: Copy + Eq,
    {
        if range.is_empty() {
            return;
        }
        let original_end_val = *self.get(range.end);
        let mut start_val = *self.get(range.start);
        f(&mut start_val);
        if Some(&start_val) == self.before(range.start) {
            self.0.remove(&range.start);
        } else {
            self.0.insert(range.start, start_val);
        }
        let mut prev = start_val;
        let mut remove = vec![];
        for (key, val) in self
            .0
            .range_mut((Bound::Excluded(range.start), Bound::Excluded(range.end)))
        {
            f(val);
            if *val == prev {
                remove.push(*key);
            }
            prev = *val;
        }
        for key in remove {
            self.0.remove(&key);
        }
        if prev == original_end_val {
            self.0.remove(&range.end);
        } else {
            self.0.insert(range.end, original_end_val);
        }
    }

    /// Returns the range and value of the last range in this RangeMap. not
    /// including the tail range.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::RangeMap;
    /// let mut map = RangeMap::default();
    /// map.insert(1..3, 1);
    /// map.insert(3..4, 2);
    /// assert_eq!(map.last_range(), Some((3..4, &2)));
    /// ```
    pub fn last_range(&self) -> Option<(Range<usize>, &T)> {
        let (start, val) = self.0.range(..self.len()).last()?;
        let end = self.len();
        Some((*start..end, val))
    }

    /// Pops the last value off the last range of this RangeMap, decreasing
    /// the length of the map by one.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::RangeMap;
    /// let mut map = RangeMap::default();
    /// map.insert(1..3, 1);
    /// map.insert(3..4, 2);
    /// map.insert(5..6, 3);
    /// assert_eq!(map.pop(), Some(3));
    /// assert_eq!(map.pop(), Some(2));
    /// assert_eq!(map.pop(), Some(1));
    /// assert_eq!(map.pop(), Some(1));
    /// assert_eq!(map.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<T>
    where
        T: Copy + Eq,
    {
        let (&k, &tail) = self.0.last_key_value().unwrap();
        let &value = self.before(k)?;
        let before = self.before(k - 1).copied();
        self.0.remove(&k);
        if Some(tail) == before {
            self.0.remove(&(k - 1));
        } else {
            self.0.insert(k - 1, tail);
        }
        Some(value)
    }

    /// Shifts the ranges of this rangemap towards 0 by the offsets in the condensation
    /// map.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::RangeMap;
    /// let mut map = RangeMap::default();
    /// map.insert(10..20, 1);
    /// map.insert(30..40, 2);
    /// map.insert(50..60, 3);
    /// let mut condensation = RangeMap::default();
    /// condensation.insert(10..30, 10);
    /// condensation.insert(30..50, 20);
    /// condensation.insert_tail(50, 30);
    /// map.shift_ranges(&condensation);
    /// assert_eq!(map.len(), 30);
    /// assert_eq!(map.iter().collect::<Vec<_>>(), vec![(0..10, 1), (10..20, 2), (20..30, 3)]);
    /// ```
    pub fn shift_ranges(&mut self, condensation: &RangeMap<usize>) {
        let elements = std::mem::take(&mut self.0);
        self.0 = elements
            .into_iter()
            .map(|(k, v)| (k - condensation.get(k), v))
            .collect();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn insert_merging_ends() {
        let mut map = RangeMap::default();
        map.insert(1..4, 3);
        map.insert(7..10, 3);
        map.insert(2..8, 3);
        assert_eq!(
            map.0.into_iter().collect::<Vec<_>>(),
            vec![(0, 0), (1, 3), (10, 0)]
        );
    }

    #[test]
    fn update_merging_ends() {
        let mut map = RangeMap::default();
        map.insert(1..4, 3);
        map.insert(4..7, 2);
        map.insert(7..10, 3);
        map.update(4..7, |x| {
            *x = 3;
        });
        assert_eq!(
            map.0.into_iter().collect::<Vec<_>>(),
            vec![(0, 0), (1, 3), (10, 0)]
        );
    }

    #[test]
    fn update_middle_of_segment() {
        let mut map = RangeMap::default();
        map.insert(1..4, 3);
        map.update(2..3, |x| {
            *x += 1;
        });
        assert_eq!(
            map.0.into_iter().collect::<Vec<_>>(),
            vec![(0, 0), (1, 3), (2, 4), (3, 3), (4, 0)]
        );
    }

    #[test]
    fn update_merge_inside() {
        let mut map = RangeMap::default();
        map.insert(1..2, 1);
        map.insert(2..3, 2);
        map.insert(3..4, 3);
        map.insert(4..5, 1);
        map.update(2..4, |x| {
            *x = 1;
        });
        assert_eq!(
            map.0.into_iter().collect::<Vec<_>>(),
            vec![(0, 0), (1, 1), (5, 0)]
        );
    }

    #[test]
    fn update_dont_merge_ends_but_merge_inside() {
        let mut map = RangeMap::default();
        map.insert(1..2, 2);
        map.insert(2..3, 2);
        map.insert(3..4, 3);
        map.insert(4..5, 2);
        map.update(2..4, |x| {
            *x = 1;
        });
        assert_eq!(
            map.0.into_iter().collect::<Vec<_>>(),
            vec![(0, 0), (1, 2), (2, 1), (4, 2), (5, 0)]
        );
    }

    #[test]
    fn insert_overlap_noop() {
        let mut map = RangeMap::default();
        map.insert(0..4, 3);
        map.insert(0..2, 3);
        assert_eq!(map.0.into_iter().collect::<Vec<_>>(), vec![(0, 3), (4, 0)]);
    }

    #[test]
    fn insert_empty_noop() {
        let mut map = RangeMap::default();
        map.insert(3..3, 3);
        assert_eq!(map.0.into_iter().collect::<Vec<_>>(), vec![(0, 0)]);
    }

    #[test]
    fn range_empty() {
        let mut map = RangeMap::default();
        map.insert(2..4, 1);
        let ranges = map.range(2..2).collect::<Vec<_>>();
        assert_eq!(ranges, vec![]);
    }

    #[test]
    fn never_remove_zero() {
        let mut map = RangeMap::default();
        map.insert(0..4, 3);
        map.remove(0..1);
        assert_eq!(
            map.0.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>(),
            vec![(0, 0), (1, 3), (4, 0)]
        );
        map.remove(0..4);
        assert_eq!(map.0.into_iter().collect::<Vec<_>>(), vec![(0, 0)]);
    }
}
