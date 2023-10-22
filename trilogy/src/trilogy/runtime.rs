use trilogy_vm::{Atom, Value};

pub struct Runtime<'a>(trilogy_vm::Execution<'a>);

impl<'a> Runtime<'a> {
    #[doc(hidden)]
    pub fn new(inner: trilogy_vm::Execution<'a>) -> Self {
        Self(inner)
    }
}

impl Runtime<'_> {
    pub fn atom(&self, tag: &str) -> Atom {
        self.0.atom(tag)
    }

    pub fn atom_anon(&self, tag: &str) -> Atom {
        self.0.atom_anon(tag)
    }

    /// The equivalent of the yield operator, allowing a native function to
    /// yield an effect.
    ///
    /// This returns an iterator which yields one item for every time a value
    /// is resumed from the effect handler.
    pub fn y<V>(&self, _value: V) -> impl Iterator<Item = Value>
    where
        Value: From<V>,
    {
        std::iter::empty()
    }
}
