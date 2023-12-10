use super::{RefCount, ReferentialEq, StructuralEq, Value};
use std::collections::HashSet;
use std::fmt::{self, Display};
use std::hash::{self, Hash};
use std::sync::Mutex;

/// A Trilogy Set value.
#[derive(Clone, Default, Debug)]
pub struct Set(RefCount<Mutex<HashSet<Value>>>);

impl Set {
    pub fn new() -> Self {
        Self::default()
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
    pub fn structural_clone(&self) -> Self {
        self.0
            .lock()
            .unwrap()
            .iter()
            .map(|v| v.structural_clone())
            .collect()
    }

    pub fn get(&self, value: &Value) -> Option<Value> {
        self.0.lock().unwrap().get(value).cloned()
    }

    pub fn has(&self, value: &Value) -> bool {
        self.0.lock().unwrap().contains(value)
    }

    pub fn insert<V>(&self, value: V) -> bool
    where
        V: Into<Value>,
    {
        self.0.lock().unwrap().insert(value.into())
    }

    pub fn remove(&self, value: &Value) -> bool {
        self.0.lock().unwrap().remove(value)
    }

    pub fn union(&self, other: &Set) {
        let mut other = other.0.lock().unwrap().clone();
        self.0.lock().unwrap().extend(other.drain());
    }

    pub fn len(&self) -> usize {
        self.0.lock().unwrap().len()
    }

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
        for item in &*self.0.lock().unwrap() {
            write!(f, "{item},")?;
        }
        write!(f, "|]")
    }
}

impl From<HashSet<Value>> for Set {
    fn from(set: HashSet<Value>) -> Self {
        Self(RefCount::new(Mutex::new(set)))
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
