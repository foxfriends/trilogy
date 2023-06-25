use super::{ReferentialEq, StructuralEq, Value};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Record(Arc<Mutex<HashMap<Value, Value>>>);

impl Eq for Record {}
impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Hash for Record {
    fn hash<H: Hasher>(&self, state: &mut H) {
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
    pub fn get(&self, key: &Value) -> Option<Value> {
        self.0.lock().unwrap().get(key).cloned()
    }

    pub fn insert(&self, key: Value, value: Value) -> Option<Value> {
        self.0.lock().unwrap().insert(key, value)
    }

    pub fn remove(&self, key: &Value) -> Option<Value> {
        self.0.lock().unwrap().remove(key)
    }
}
