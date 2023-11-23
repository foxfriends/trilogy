use super::{ReferentialEq, StructuralEq, Value};
use std::collections::HashMap;
use std::fmt::{self, Display};
use std::hash::{self, Hash};
use std::sync::{Arc, Mutex};

/// An instance of a Trilogy Record.
///
/// Record instances in Trilogy are mutable and shared. Therefore, modifications made to such an
/// Record of this type will be reflected on all references to the underlying instance.
///
/// ```
/// # use trilogy_vm::runtime::{Record, Value};
/// # use std::collections::HashMap;
/// let record = Record::new();
/// record.insert("string", "hello");
/// record.insert("number", 123);
/// let copy = record.clone();
/// record.insert("boolean", false);
/// assert_eq!(copy.get(&Value::from("boolean")), Some(Value::from(false)));
/// ```
///
/// A [`Clone`][] of an Record will still be pointing to the same instance. To get a
/// new instance, see [`shallow_clone`][Record::shallow_clone] or [`structural_clone`][Record::structural_clone].
///
/// ```
/// # use trilogy_vm::runtime::Record;
/// let record = Record::new();
/// assert_eq!(record, record.clone());
/// assert_ne!(record, Record::new());
/// ```
#[derive(Clone, Default, Debug)]
pub struct Record(Arc<Mutex<HashMap<Value, Value>>>);

impl Eq for Record {}
impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Hash for Record {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
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
        // Check pointer equality first: if it's the same instance, we can't
        // do structural comparison like this because of the locks
        if Arc::ptr_eq(&self.0, &other.0) {
            return true;
        }
        let Ok(lhs) = self.0.lock() else { return false };
        let Ok(rhs) = other.0.lock() else {
            return false;
        };
        lhs.eq(&*rhs)
    }
}

impl Record {
    /// Creates a new empty record instance.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::Record;
    /// let record = Record::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Performs a shallow clone of the record, returning a new record
    /// instance over the same elements. Keys and values will not be cloned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Record, Value};
    /// let record = Record::new();
    /// record.insert(0, Record::new());
    /// let clone = record.shallow_clone();
    /// assert_ne!(record, clone);
    /// assert_eq!(record.get(&Value::from(0)), clone.get(&Value::from(0)));
    /// ```
    pub fn shallow_clone(&self) -> Self {
        Self::from(self.0.lock().unwrap().clone())
    }

    /// Performs a deep structural clone of the record, returning a completely
    /// fresh copy of the same value. All keys and values are recursively
    /// structural cloned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Record, Value};
    /// let record = Record::new();
    /// record.insert(0, Record::new());
    /// let clone = record.structural_clone();
    /// assert_ne!(record, clone);
    /// assert_ne!(record.get(&Value::from(0)), clone.get(&Value::from(0)));
    /// ```
    pub fn structural_clone(&self) -> Self {
        self.0
            .lock()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.structural_clone(), v.structural_clone()))
            .collect()
    }

    /// Gets the value from a particular key of this record. Returns None
    /// if the record does not contain the key.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Record, Value};
    /// let record = Record::new();
    /// record.insert(0, "hello");
    /// assert_eq!(record.get(&Value::from(0)), Some(Value::from("hello")));
    /// assert_eq!(record.get(&Value::from(1)), None);
    /// ```
    pub fn get(&self, key: &Value) -> Option<Value> {
        self.0.lock().unwrap().get(key).cloned()
    }

    /// Returns true of this Record contains a given key.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Record, Value};
    /// let record = Record::new();
    /// record.insert(0, "hello");
    /// assert!(record.contains_key(&Value::from(0)));
    /// assert!(!record.contains_key(&Value::from(1)));
    /// ```
    pub fn contains_key(&self, key: &Value) -> bool {
        self.0.lock().unwrap().contains_key(key)
    }

    /// Inserts a key-value pair into this record.
    ///
    /// Note that since Trilogy records are internally mutable, you do not
    /// need a mutable reference to an Record to modify its contents.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Record, Value};
    /// let record = Record::new();
    /// assert!(!record.contains_key(&Value::from(0)));
    /// record.insert(0, "hello");
    /// assert!(record.contains_key(&Value::from(0)));
    /// ```
    pub fn insert<K, V>(&self, key: K, value: V) -> Option<Value>
    where
        K: Into<Value>,
        V: Into<Value>,
    {
        self.0.lock().unwrap().insert(key.into(), value.into())
    }

    /// Merges other and self by adding the key-value pairs of other to self.
    ///
    /// Values associated with found in both records will be taken from `other`.
    /// The other record and its values are unmodified.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Record, Value};
    /// let first = Record::new();
    /// first.insert(0, 0);
    /// first.insert(1, 0);
    /// let second = Record::new();
    /// second.insert(1, 1);
    /// second.insert(2, 2);
    /// first.union(&second);
    /// assert_eq!(first.get(&Value::from(0)), Some(Value::from(0)));
    /// assert_eq!(first.get(&Value::from(1)), Some(Value::from(1)));
    /// assert_eq!(first.get(&Value::from(2)), Some(Value::from(2)));
    /// assert_eq!(second.len(), 2);
    /// ```
    pub fn union(&self, other: &Record) {
        let mut other = other.0.lock().unwrap().clone();
        self.0.lock().unwrap().extend(other.drain());
    }

    /// Removes the key-value pair associated with a key of this Record. Returns the removed
    /// value, or None if the key is not in the Record.
    ///
    /// If a value was removed, the length of the record will be decreased by 1.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Record, Value};
    /// let record = Record::new();
    /// record.insert(0, 0);
    /// assert_eq!(record.remove(&Value::from(0)), Some(Value::from(0)));
    /// assert_eq!(record.len(), 0);
    /// ```
    pub fn remove(&self, key: &Value) -> Option<Value> {
        self.0.lock().unwrap().remove(key)
    }

    /// Returns the number of key-value pairs in the Record.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Record, Value};
    /// let record = Record::new();
    /// record.insert(0, 0);
    /// record.insert(1, 1);
    /// assert_eq!(record.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.0.lock().unwrap().len()
    }

    /// Returns true if the Record contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Record, Value};
    /// let record = Record::new();
    /// assert!(record.is_empty());
    ///
    /// record.insert(1, 1);
    /// assert!(!record.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.lock().unwrap().is_empty()
    }

    /// Converts this Record to a native Rust `HashMap`.
    ///
    /// Note that as the returned HashMap is not a Record instance, modifications made to
    /// it will not be reflected on the underlying value. Values in this `HashMap` however
    /// may still be Trilogy instances.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Record, Value};
    /// # use std::collections::HashMap;
    /// let record = Record::new();
    /// record.insert(0, 0);
    /// let mut map = HashMap::new();
    /// map.insert(Value::from(0), Value::from(0));
    /// assert_eq!(record.to_map(), map);
    /// ```
    pub fn to_map(&self) -> HashMap<Value, Value> {
        self.0.lock().unwrap().clone()
    }
}

impl IntoIterator for &'_ Record {
    type Item = (Value, Value);
    type IntoIter = <HashMap<Value, Value> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.lock().unwrap().clone().into_iter()
    }
}

impl Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{|")?;
        for (key, value) in &*self.0.lock().unwrap() {
            write!(f, "{key}=>{value},")?;
        }
        write!(f, "|}}")
    }
}

impl From<HashMap<Value, Value>> for Record {
    fn from(value: HashMap<Value, Value>) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }
}

impl From<Record> for HashMap<Value, Value> {
    fn from(value: Record) -> Self {
        (*value.0.lock().unwrap()).clone()
    }
}

impl FromIterator<(Value, Value)> for Record {
    fn from_iter<T: IntoIterator<Item = (Value, Value)>>(iter: T) -> Self {
        Self::from(HashMap::from_iter(iter))
    }
}
