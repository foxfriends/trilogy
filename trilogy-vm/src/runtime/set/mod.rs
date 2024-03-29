use super::{RefCount, ReferentialEq, StructuralEq, Value};
use std::collections::HashSet;
use std::fmt::{self, Display};
use std::hash::{self, Hash};
use std::sync::Mutex;

mod inner;

/// A Trilogy Set value.
#[derive(Clone, Default, Debug)]
pub struct Set(RefCount<Mutex<inner::SetInner>>);

#[cfg(feature = "serde")]
impl serde::Serialize for Set {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let inner = self.0.lock().unwrap();
        let mut seq = serializer.serialize_seq(Some(inner.len()))?;
        for e in &**inner {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

impl Set {
    /// Creates a new empty set instance.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::Set;
    /// let set = Set::new();
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the ID of the underlying set instance. This ID will remain
    /// stable for the lifetime of each set instance, and is unique per
    /// instance.
    ///
    /// Note that set instances may be reused, so if an instance is conceptually
    /// discarded in the Trilogy program, that same instance may be reclaimed by
    /// the runtime to reuse its allocation. The Trilogy program should therefore
    /// never expect to use this ID internally.
    pub fn id(&self) -> usize {
        RefCount::as_ptr(&self.0) as usize
    }

    /// Performs a shallow clone of the set, returning a new set
    /// instance over the same elements. The values of the set are not cloned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Set, Value};
    /// let set = Set::new();
    /// let inner = Value::from(Set::new());
    /// set.insert(inner.clone());
    /// let clone = set.shallow_clone();
    /// assert_ne!(set, clone);
    /// assert!(set.has(&inner));
    /// assert!(clone.has(&inner));
    /// ```
    #[inline]
    pub fn shallow_clone(&self) -> Self {
        Self::from(self.0.lock().unwrap().clone())
    }

    /// Performs a deep structural clone of the set, returning a completely
    /// fresh copy of the same value. All values are recursively structural
    /// cloned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Set, Value};
    /// let set = Set::new();
    /// let inner = Value::from(Set::new());
    /// set.insert(inner.clone());
    /// let clone = set.structural_clone();
    /// assert_ne!(set, clone);
    /// assert!(set.has(&inner));
    /// assert!(!clone.has(&inner));
    /// ```
    #[inline]
    pub fn structural_clone(&self) -> Self {
        self.0
            .lock()
            .unwrap()
            .iter()
            .map(|v| v.structural_clone())
            .collect()
    }

    /// Returns true of this Set contains a given value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Set, Value};
    /// let set = Set::new();
    /// set.insert("hello");
    /// assert!(set.has(&Value::from("hello")));
    /// assert!(!set.has(&Value::from("world")));
    /// ```
    #[inline]
    pub fn has(&self, value: &Value) -> bool {
        self.0.lock().unwrap().contains(value)
    }

    /// Adds a value to the set.
    ///
    /// Returns whether the value was newly inserted. That is:
    ///
    /// * If the set did not previously contain this value, `true` is returned.
    /// * If the set already contained this value, `false` is returned,
    ///   and the set is not modified: original value is not replaced,
    ///   and the value passed as argument is dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Set, Value};
    /// let set = Set::new();
    /// assert_eq!(set.insert(2), true);
    /// assert_eq!(set.insert(2), false);
    /// assert_eq!(set.len(), 1);
    /// ```
    #[inline]
    pub fn insert<V>(&self, value: V) -> bool
    where
        V: Into<Value>,
    {
        self.0.lock().unwrap().insert(value.into())
    }

    /// Removes a value from the set. Returns whether the value was
    /// present in the set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use trilogy_vm::runtime::{Set, Value};
    /// let set = Set::new();
    /// set.insert(2);
    /// assert_eq!(set.remove(&Value::from(2)), true);
    /// assert_eq!(set.remove(&Value::from(2)), false);
    /// ```
    #[inline]
    pub fn remove(&self, value: &Value) -> bool {
        self.0.lock().unwrap().remove(value)
    }

    /// Merges other and self by adding the values pairs of `other` to `self`.
    ///
    /// Values associated with found in both sets will be taken from `other`.
    /// The other set and its values are unmodified.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use trilogy_vm::runtime::{Set, Value};
    /// let first = Set::new();
    /// first.insert(2);
    /// first.insert(3);
    /// let second = Set::new();
    /// second.insert(3);
    /// second.insert(4);
    /// first.union(&second);
    /// assert_eq!(first.len(), 3);
    /// assert_eq!(first.has(&Value::from(3)), true);
    /// ```
    #[inline]
    pub fn union(&self, other: &Set) {
        let mut other = other.0.lock().unwrap().clone();
        self.0.lock().unwrap().extend(other.drain());
    }

    /// Merges other and self by adding the values pairs of `other` to `self`.
    /// In the case that this is the only reference to `other`, avoids cloning
    /// the contained elements.
    ///
    /// Values associated with found in both sets will be taken from `other`.
    /// The other set and its values are unmodified.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use trilogy_vm::runtime::{Set, Value};
    /// let first = Set::new();
    /// first.insert(2);
    /// first.insert(3);
    /// let second = Set::new();
    /// second.insert(3);
    /// second.insert(4);
    /// first.union_from(second);
    /// assert_eq!(first.len(), 3);
    /// assert_eq!(first.has(&Value::from(3)), true);
    /// ```
    #[inline]
    pub fn union_from(&self, other: Set) {
        let mut other = match RefCount::try_unwrap(other.0) {
            Ok(other) => other,
            Err(other) => return self.union(&Set(other)),
        };
        self.0
            .lock()
            .unwrap()
            .extend(other.get_mut().unwrap().drain());
    }

    /// Returns the number of elements in this set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Set, Value};
    /// let set = Set::new();
    /// set.insert(3);
    /// set.insert(4);
    /// assert_eq!(set.len(), 2);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.0.lock().unwrap().len()
    }

    /// Returns true if the set contains no elements, or false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Set, Value};
    /// let set = Set::new();
    /// assert_eq!(set.is_empty(), true);
    /// set.insert(3);
    /// set.insert(4);
    /// assert_eq!(set.is_empty(), false);
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.lock().unwrap().is_empty()
    }
}

impl IntoIterator for &'_ Set {
    type Item = Value;
    type IntoIter = <HashSet<Value, Value> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.lock().unwrap().clone().into_iter()
    }
}

impl Eq for Set {}
impl PartialEq for Set {
    fn eq(&self, other: &Self) -> bool {
        RefCount::ptr_eq(&self.0, &other.0)
    }
}
impl Hash for Set {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        RefCount::as_ptr(&self.0).hash(state);
    }
}

impl ReferentialEq for Set {
    fn eq(&self, other: &Self) -> bool {
        RefCount::ptr_eq(&self.0, &other.0)
    }
}

impl StructuralEq for Set {
    fn eq(&self, other: &Self) -> bool {
        // Check pointer equality first: if it's the same instance, we can't
        // do structural comparison like this because of the locks
        if RefCount::ptr_eq(&self.0, &other.0) {
            return true;
        }
        let Ok(lhs) = self.0.lock() else { return false };
        let Ok(rhs) = other.0.lock() else {
            return false;
        };
        // TODO: this is not super well defined...
        lhs.eq(&*rhs)
    }
}

impl Display for Set {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[|")?;
        for item in &**self.0.lock().unwrap() {
            write!(f, "{item},")?;
        }
        write!(f, "|]")
    }
}

impl From<HashSet<Value>> for Set {
    fn from(set: HashSet<Value>) -> Self {
        Self(RefCount::new(Mutex::new(inner::SetInner::new(set))))
    }
}

impl From<Set> for HashSet<Value> {
    fn from(value: Set) -> Self {
        (*value.0.lock().unwrap()).clone()
    }
}

impl FromIterator<Value> for Set {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        Self::from(HashSet::from_iter(iter))
    }
}
