use super::{ReferentialEq, StructuralEq, Value};
use std::collections::HashSet;
use std::fmt::{self, Display};
use std::hash::{self, Hash};
use std::sync::{Arc, Mutex};

/// A Trilogy Set value.
#[derive(Clone, Default, Debug)]
pub struct Set(Arc<Mutex<HashSet<Value>>>);

impl Set {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn structural_clone(&self) -> Self {
        Self::from(self.0.lock().unwrap().clone())
    }

    pub fn get(&self, value: &Value) -> Option<Value> {
        self.0.lock().unwrap().get(value).cloned()
    }

    pub fn has(&self, value: &Value) -> bool {
        self.0.lock().unwrap().contains(value)
    }

    pub fn insert(&self, value: Value) -> bool {
        self.0.lock().unwrap().insert(value)
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
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Hash for Set {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

impl ReferentialEq for Set {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl StructuralEq for Set {
    fn eq(&self, other: &Self) -> bool {
        // Check pointer equality first: if it's the same instance, we can't
        // do structural comparison like this because of the locks
        if Arc::ptr_eq(&self.0, &other.0) {
            return true;
        }
        let Ok(lhs) = self.0.lock() else { return false };
        let Ok(rhs) = other.0.lock() else {
            return false;
        };
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
        Self(Arc::new(Mutex::new(set)))
    }
}
