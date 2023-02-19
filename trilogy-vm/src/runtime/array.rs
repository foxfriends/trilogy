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
