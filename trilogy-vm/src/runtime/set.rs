use super::{ReferentialEq, StructuralEq, Value};
use std::collections::HashSet;
use std::fmt::{self, Display};
use std::hash::{self, Hash};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Set(Arc<Mutex<HashSet<Value>>>);

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
        let Ok(lhs) = self.0.lock() else {
            return false
        };
        let Ok(rhs) = other.0.lock() else {
            return false
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
