use std::borrow::Borrow;
use std::cmp::PartialEq;
use std::collections::HashSet;
use std::fmt::{self, Display};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Atom(Arc<String>);

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

#[derive(Clone, Eq, PartialEq, Hash)]
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

#[derive(Clone, Default)]
pub(crate) struct AtomInterner(HashSet<AtomRaw>);

impl AtomInterner {
    pub fn intern(&mut self, string: &str) -> Atom {
        if let Some(arc) = self.0.get(string) {
            (*arc).clone().into()
        } else {
            let arc = Arc::new(string.to_owned());
            self.0.insert(AtomRaw(arc.clone()));
            Atom(arc)
        }
    }

    pub fn lookup(&self, string: &str) -> Option<Atom> {
        let arc = self.0.get(string)?;
        Some((*arc).clone().into())
    }
}
