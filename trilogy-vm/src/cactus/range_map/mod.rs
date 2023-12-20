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
/// This map is specialized for `usize` ranges to `usize` values. It could
/// be expanded to any `Ord` key and any `copy` (or `Clone`) value if desired.
#[derive(Clone, Debug)]
pub struct RangeMap(BTreeMap<usize, usize>);

impl Default for RangeMap {
    fn default() -> Self {
        let mut map = BTreeMap::new();
        map.insert(0, 0);
        RangeMap(map)
    }
}

impl RangeMap {
    /// Creates a new empty `RangeMap`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::RangeMap;
    /// let map = RangeMap::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// An iterator over all ranges that are contained in this RangeMap.
    ///
    /// A range is "contained" if its value is not zero.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::cactus::RangeMap;
    /// let mut map = RangeMap::default();
    /// map.insert(2..4, 1);
    /// map.insert(6..8, 2);
    /// let ranges = map.ranges().collect::<Vec<_>>();
    /// assert_eq!(ranges, vec![(2..4), (6..8)]);
    /// ```
    #[inline]
    pub fn ranges(&self) -> impl Iterator<Item = Range<usize>> + '_ {
        self.0
            .iter()
            .peekable()
            .pairwise()
            .filter(|((_, v), _)| **v > 0)
            .map(|((s, _), (e, _))| (*s..*e))
    }

    /// An iterator over all ranges in this RangeMap.
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
    /// let ranges = map.ranges().collect::<Vec<_>>();
    /// assert_eq!(ranges, vec![(2..4), (6..8)]);
    /// ```
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (Range<usize>, usize)> + '_ {
        self.0
            .iter()
            .peekable()
            .pairwise()
            .map(|((s, v), (e, _))| (*s..*e, *v))
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
    pub fn range(&self, range: Range<usize>) -> impl Iterator<Item = (Range<usize>, usize)> + '_ {
        if range.is_empty() {
            return Box::new(std::iter::empty()) as Box<dyn Iterator<Item = (Range<usize>, usize)>>;
        }
        let start_val = self.get(range.start);
        Box::new(
            std::iter::once((range.start, start_val))
                .chain(
                    self.0
                        .range((Bound::Excluded(range.start), Bound::Excluded(range.end)))
                        .map(|(s, v)| (*s, *v)),
                )
                .chain(std::iter::once((range.end, 0)))
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
    /// assert_eq!(map.get(0), 0);
    /// assert_eq!(map.get(2), 1);
    /// assert_eq!(map.get(7), 2);
    /// assert_eq!(map.get(8), 0);
    /// ```
    #[inline]
    pub fn get(&self, key: usize) -> usize {
        *self
            .0
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
    /// assert_eq!(map.before(2), Some(0));
    /// assert_eq!(map.before(7), Some(2));
    /// assert_eq!(map.before(8), Some(2));
    /// assert_eq!(map.before(9), Some(0));
    /// ```
    #[inline]
    pub fn before(&self, key: usize) -> Option<usize> {
        self.0
            .range((Bound::Unbounded, Bound::Excluded(key)))
            .last()
            .map(|kv| *kv.1)
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
    /// assert_eq!(map.get(0), 0);
    /// assert_eq!(map.get(1), 3);
    /// assert_eq!(map.get(2), 3);
    /// assert_eq!(map.get(3), 3);
    /// assert_eq!(map.get(4), 1);
    /// assert_eq!(map.get(5), 1);
    /// assert_eq!(map.get(6), 2);
    /// assert_eq!(map.get(7), 1);
    /// assert_eq!(map.get(8), 0);
    /// ```
    #[inline]
    pub fn insert(&mut self, range: Range<usize>, value: usize) {
        if range.is_empty() {
            return;
        }
        let before = self.before(range.start);
        let after = self.get(range.end);
        let keys_to_remove = self
            .0
            .range((Bound::Excluded(range.start), Bound::Excluded(range.end)))
            .map(|(k, _)| *k)
            .collect::<Vec<_>>();
        for key in keys_to_remove {
            self.0.remove(&key);
        }
        if before == Some(value) {
            self.0.remove(&range.start);
        } else {
            self.0.insert(range.start, value);
        }
        if after == value {
            self.0.remove(&range.end);
        } else {
            self.0.insert(range.end, after);
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
    /// assert_eq!(map.get(0), 0);
    /// assert_eq!(map.get(1), 0);
    /// assert_eq!(map.get(2), 0);
    /// assert_eq!(map.get(3), 0);
    /// assert_eq!(map.get(4), 1);
    /// assert_eq!(map.get(5), 1);
    /// assert_eq!(map.get(6), 0);
    /// ```
    #[inline]
    pub fn remove(&mut self, range: Range<usize>) {
        if range.is_empty() {
            return;
        }
        self.insert(range, 0);
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
    /// assert_eq!(map.get(0), 2);
    /// assert_eq!(map.get(1), 2);
    /// assert_eq!(map.get(2), 4);
    /// assert_eq!(map.get(3), 4);
    /// assert_eq!(map.get(4), 6);
    /// assert_eq!(map.get(5), 6);
    /// assert_eq!(map.get(6), 2);
    /// assert_eq!(map.get(7), 0);
    /// ```
    #[inline]
    pub fn update<F: Fn(&mut usize)>(&mut self, range: Range<usize>, f: F) {
        if range.is_empty() {
            return;
        }
        let mut start_val = self.get(range.start);
        f(&mut start_val);
        if Some(start_val) == self.before(range.start) {
            self.0.remove(&range.start);
        } else {
            self.0.insert(range.start, start_val);
        }
        let original_end_val = self.get(range.end);
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn insert_merging_ends() {
        let mut map = RangeMap::new();
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
        let mut map = RangeMap::new();
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
    fn update_merge_inside() {
        let mut map = RangeMap::new();
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
        let mut map = RangeMap::new();
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
        let mut map = RangeMap::new();
        map.insert(0..4, 3);
        map.insert(0..2, 3);
        assert_eq!(map.0.into_iter().collect::<Vec<_>>(), vec![(0, 3), (4, 0)]);
    }

    #[test]
    fn insert_empty_noop() {
        let mut map = RangeMap::new();
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
    fn ranges_empty() {
        let map = RangeMap::default();
        let ranges = map.ranges().collect::<Vec<_>>();
        assert_eq!(ranges, vec![]);
    }

    #[test]
    fn never_remove_zero() {
        let mut map = RangeMap::new();
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
