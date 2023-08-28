use super::{ReferentialEq, StructuralEq, Value};
use std::fmt::{self, Display};
use std::hash::{self, Hash};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default, Debug)]
pub struct Array(Arc<Mutex<Vec<Value>>>);

impl Eq for Array {}

impl PartialEq for Array {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Hash for Array {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
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
        let Ok(lhs) = self.0.lock() else { return false };
        let Ok(rhs) = other.0.lock() else {
            return false;
        };
        lhs.eq(&*rhs)
    }
}

impl IntoIterator for &'_ Array {
    type Item = Value;
    type IntoIter = <Vec<Value> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.lock().unwrap().clone().into_iter()
    }
}

impl PartialOrd for Array {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let Ok(lhs) = self.0.lock() else { return None };
        let Ok(rhs) = other.0.lock() else { return None };
        lhs.partial_cmp(&*rhs)
    }
}

impl Array {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn structural_clone(&self) -> Self {
        Self::from(self.0.lock().unwrap().clone())
    }

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

    pub fn contains(&self, value: &Value) -> bool {
        self.0.lock().unwrap().contains(value)
    }

    pub fn remove(&self, index: usize) -> Option<Value> {
        let mut array = self.0.lock().unwrap();
        if index <= array.len() {
            Some(array.remove(index))
        } else {
            None
        }
    }

    pub fn push(&self, value: Value) {
        self.0.lock().unwrap().push(value);
    }

    pub fn append(&self, other: &Array) {
        let mut other = other.0.lock().unwrap().clone();
        self.0.lock().unwrap().append(&mut other);
    }

    pub fn len(&self) -> usize {
        self.0.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.lock().unwrap().is_empty()
    }

    pub fn range<I>(&self, range: I) -> Self
    where
        Vec<Value>: std::ops::Index<I, Output = [Value]>,
    {
        self.0.lock().unwrap()[range].to_vec().into()
    }
}

impl Display for Array {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for item in &*self.0.lock().unwrap() {
            write!(f, "{item},")?;
        }
        write!(f, "]")
    }
}

impl From<Vec<Value>> for Array {
    fn from(value: Vec<Value>) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }
}
