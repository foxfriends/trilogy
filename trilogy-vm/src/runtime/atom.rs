use std::borrow::Borrow;
use std::cmp::PartialEq;
use std::collections::HashSet;
use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

/// A Trilogy Atom value.
#[derive(Clone)]
pub struct Atom(Arc<String>);

impl Debug for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Atom({}#{:#x})", self.0, self.0.as_ptr() as usize)
    }
}

impl Atom {
    pub(crate) fn new_unique(label: String) -> Self {
        Self(Arc::new(label))
    }
}

impl Eq for Atom {}
impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Hash for Atom {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state);
    }
}

impl Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "'{}", self.0)
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
struct AtomRaw(Arc<String>);

impl From<AtomRaw> for Atom {
    fn from(value: AtomRaw) -> Self {
        Self(value.0)
    }
}

impl Borrow<str> for AtomRaw {
    fn borrow(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Clone, Default, Debug)]
pub(crate) struct AtomInterner(Arc<Mutex<HashSet<AtomRaw>>>);

impl AtomInterner {
    pub fn intern(&self, string: &str) -> Atom {
        let mut contents = self.0.lock().unwrap();
        if let Some(arc) = contents.get(string) {
            (*arc).clone().into()
        } else {
            let arc = Arc::new(string.to_owned());
            contents.insert(AtomRaw(arc.clone()));
            Atom(arc)
        }
    }
}
