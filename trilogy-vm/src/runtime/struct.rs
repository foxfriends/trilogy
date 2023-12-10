use super::{Atom, RefCount, ReferentialEq, StructuralEq, Value};
use std::fmt::{self, Display};

/// A Trilogy Struct value.
///
/// In Trilogy, a "struct" is a single other `Value` tagged with an Atom, not quite
/// the same as a struct you might see in languages such as Rust.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Struct {
    name: Atom,
    value: RefCount<Value>,
}

impl StructuralEq for Struct {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && StructuralEq::eq(&*self.value, &*other.value)
    }
}

impl ReferentialEq for Struct {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && ReferentialEq::eq(&*self.value, &*other.value)
    }
}

impl Struct {
    /// Creates a struct value using the given atom and value.
    ///
    /// Atoms can be created using methods provided by a VM instance, such
    /// as [`VirtualMachine::atom`][crate::VirtualMachine::atom] or [`Execution::atom`][crate::Execution::atom].
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::{VirtualMachine, Struct, Value};
    /// # let vm = VirtualMachine::new();
    /// let tag = vm.atom("number");
    /// let struct_value = Struct::new(tag, 3);
    /// ```
    pub fn new<V>(name: Atom, value: V) -> Self
    where
        Value: From<V>,
    {
        Self {
            name,
            value: RefCount::new(value.into()),
        }
    }

    /// Returns the name (atom) of this struct value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::{VirtualMachine, Struct, Value};
    /// # let vm = VirtualMachine::new();
    /// let tag = vm.atom("number");
    /// let struct_value = Struct::new(tag.clone(), 3);
    /// assert_eq!(struct_value.name(), tag);
    /// ```
    pub fn name(&self) -> Atom {
        self.name.clone()
    }

    /// Returns a reference to the contained value of this struct.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::{VirtualMachine, Struct, Value};
    /// # let vm = VirtualMachine::new();
    /// let tag = vm.atom("number");
    /// let struct_value = Struct::new(tag.clone(), 3);
    /// assert_eq!(struct_value.value(), &Value::from(3));
    /// ```
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// Deconstructs the struct into its name and value as a tuple.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::{VirtualMachine, Struct, Value};
    /// # let vm = VirtualMachine::new();
    /// let tag = vm.atom("number");
    /// let struct_value = Struct::new(tag.clone(), 3);
    /// assert_eq!(struct_value.destruct(), (tag, Value::from(3)));
    /// ```
    pub fn destruct(self) -> (Atom, Value) {
        (self.name, (*self.value).clone())
    }

    /// Returns the contained value of this struct, consuming the struct.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::{VirtualMachine, Struct, Value};
    /// # let vm = VirtualMachine::new();
    /// let tag = vm.atom("number");
    /// let struct_value = Struct::new(tag, 3);
    /// assert_eq!(struct_value.into_value(), Value::from(3));
    /// ```
    pub fn into_value(self) -> Value {
        (*self.value).clone()
    }
}

impl PartialOrd for Struct {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.name == other.name {
            (*self.value).partial_cmp(&*other.value)
        } else {
            None
        }
    }
}

impl Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.name, self.value)
    }
}
