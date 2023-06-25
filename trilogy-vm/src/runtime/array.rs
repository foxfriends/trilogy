use super::{ReferentialEq, StructuralEq, Value};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Array(Arc<Mutex<Vec<Value>>>);

impl Eq for Array {}
impl PartialEq for Array {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Hash for Array {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

impl ReferentialEq for Array {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl StructuralEq for Array {
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

impl Array {
    pub fn get(&self, index: usize) -> Option<Value> {
        self.0.lock().unwrap().get(index).cloned()
    }

    pub fn set(&self, index: usize, value: Value) {
        let mut array = self.0.lock().unwrap();
        if array.len() <= index {
            array.resize(index + 1, Value::Unit);
        }
        array[index] = value;
    }

    pub fn remove(&self, index: usize) -> Option<Value> {
        let mut array = self.0.lock().unwrap();
        if index <= array.len() {
            Some(array.remove(index))
        } else {
            None
        }
    }
}
