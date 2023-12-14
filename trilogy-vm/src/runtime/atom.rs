use std::borrow::Borrow;
use std::cmp::PartialEq;
use std::collections::HashSet;
use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

/// A Trilogy Atom value.
///
/// Atoms are similar to what might also be called a symbol in other languages.
/// In particular Atoms are comparable in constant time, unlike using a string
/// of the same contents.
///
/// Two atoms created from the same literal (on the same VM) will be equal.
///
/// ```
/// # use trilogy_vm::VirtualMachine;
/// # let vm = VirtualMachine::default();
/// // Atoms created with the `vm.atom()` function are the same as atom literals created
/// // in source code.
/// let hello = vm.atom("hello");
/// let hello_again = vm.atom("hello");
/// let world = vm.atom("world");
/// assert_eq!(hello, hello_again);
/// assert_ne!(hello, world);
/// ```
///
/// It is possible for atoms to be created anonymously as well. Such atoms still
/// have a string representation, but they will *not* be equal to another atom
/// created separately, even if that string representation is the same. Such
/// unique atoms cannot be created from literals.
///
/// ```
/// # use trilogy_vm::VirtualMachine;
/// # let vm = VirtualMachine::default();
/// let hello = vm.atom("hello");
/// let hello_2 = vm.atom_anon("hello");
/// let hello_3 = vm.atom_anon("hello");
/// assert_ne!(hello, hello_2);
/// assert_ne!(hello_2, hello_3);
/// ```
#[derive(Clone)]
pub struct Atom(Arc<String>);

impl Debug for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Atom({}#{:#x})", self.0, self.0.as_ptr() as usize)
    }
}

impl Atom {
    #[inline]
    pub(crate) fn new_unique(label: String) -> Self {
        Self(Arc::new(label))
    }
}

impl Eq for Atom {}
impl PartialEq for Atom {
    #[inline]
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

impl AsRef<str> for Atom {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
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
