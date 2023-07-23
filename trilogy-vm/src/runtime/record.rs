use super::{ReferentialEq, StructuralEq, Value};
use std::collections::HashMap;
use std::fmt::{self, Display};
use std::hash::{self, Hash};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default, Debug)]
pub struct Record(Arc<Mutex<HashMap<Value, Value>>>);

impl Eq for Record {}
impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Hash for Record {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

impl ReferentialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl StructuralEq for Record {
    fn eq(&self, other: &Self) -> bool {
        let Ok(lhs) = self.0.lock() else {
            return false
        };
        let Ok(rhs) = other.0.lock() else {
            return false
        };
        lhs.eq(&*rhs)
    }
}

impl Record {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn structural_clone(&self) -> Self {
        Self::from(self.0.lock().unwrap().clone())
    }

    pub fn get(&self, key: &Value) -> Option<Value> {
        self.0.lock().unwrap().get(key).cloned()
    }

    pub fn contains_key(&self, key: &Value) -> bool {
        self.0.lock().unwrap().contains_key(key)
    }

    pub fn insert(&self, key: Value, value: Value) -> Option<Value> {
        self.0.lock().unwrap().insert(key, value)
    }

    pub fn remove(&self, key: &Value) -> Option<Value> {
        self.0.lock().unwrap().remove(key)
    }

    pub fn len(&self) -> usize {
        self.0.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.lock().unwrap().is_empty()
    }
}

impl IntoIterator for &'_ Record {
    type Item = (Value, Value);
    type IntoIter = <HashMap<Value, Value> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.lock().unwrap().clone().into_iter()
    }
}

impl Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{|")?;
        for (key, value) in &*self.0.lock().unwrap() {
            write!(f, "{key}=>{value},")?;
        }
        write!(f, "|}}")
    }
}

impl From<HashMap<Value, Value>> for Record {
    fn from(value: HashMap<Value, Value>) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }
}
