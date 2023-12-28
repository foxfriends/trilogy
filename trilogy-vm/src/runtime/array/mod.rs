use super::{RefCount, ReferentialEq, StructuralEq, Value};
use std::fmt::{self, Debug, Display};
use std::hash::{self, Hash};
use std::sync::Mutex;

mod inner;

/// An instance of a Trilogy Array.
///
/// Array instances in Trilogy are mutable and shared. Therefore, modifications made to such an
/// Array of this type will be reflected on all references to the underlying instance.
///
/// ```
/// # use trilogy_vm::runtime::{Array, Value};
/// let array = Array::new();
/// array.push(1);
/// array.push(2);
/// let copy = array.clone();
/// array.push(3);
/// assert_eq!(copy.get(2), Some(Value::from(3)));
/// ```
///
/// A [`Clone`][] of an Array will still be pointing to the same instance. To get a
/// new instance, see [`shallow_clone`][Array::shallow_clone] or [`structural_clone`][Array::structural_clone].
///
/// ```
/// # use trilogy_vm::runtime::Array;
/// let array = Array::new();
/// assert_eq!(array, array.clone());
/// assert_ne!(array, Array::new());
/// ```
#[derive(Clone, Default)]
pub struct Array(RefCount<Mutex<inner::ArrayInner>>);

impl Debug for Array {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.0.lock().unwrap();
        f.debug_tuple("Array").field(&*inner).finish()
    }
}

impl Eq for Array {}

/// Equality for Arrays is based on referential equality of the underlying Array
/// instance.
impl PartialEq for Array {
    fn eq(&self, other: &Self) -> bool {
        RefCount::ptr_eq(&self.0, &other.0)
    }
}

impl Hash for Array {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        RefCount::as_ptr(&self.0).hash(state);
    }
}

impl ReferentialEq for Array {
    fn eq(&self, other: &Self) -> bool {
        RefCount::ptr_eq(&self.0, &other.0)
    }
}

impl StructuralEq for Array {
    fn eq(&self, other: &Self) -> bool {
        // Check pointer equality first: if it's the same instance, we can't
        // do structural comparison like this because of the locks
        if RefCount::ptr_eq(&self.0, &other.0) {
            return true;
        }
        let Ok(lhs) = self.0.lock() else { return false };
        let Ok(rhs) = other.0.lock() else {
            return false;
        };
        if lhs.len() != rhs.len() {
            return false;
        }
        for i in 0..lhs.len() {
            if !StructuralEq::eq(&lhs[i], &rhs[i]) {
                return false;
            }
        }
        true
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
    /// Creates a new empty array instance.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::Array;
    /// let array = Array::new();
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the ID of the underlying array instance. This ID will remain
    /// stable for the lifetime of each array instance, and is unique per
    /// instance.
    ///
    /// Note that array instances may be reused, so if an instance is conceptually
    /// discarded in the Trilogy program, that same instance may be reclaimed by
    /// the runtime to reuse its allocation. The Trilogy program should therefore
    /// never expect to use this ID internally.
    pub fn id(&self) -> usize {
        RefCount::as_ptr(&self.0) as usize
    }

    /// Performs a shallow clone of the array, returning a new array
    /// instance over the same elements. The elements will not be cloned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Array, Value};
    /// let array = Array::new();
    /// array.push(Array::new());
    /// let clone = array.shallow_clone();
    /// assert_ne!(array, clone);
    /// assert_eq!(array.get(0), clone.get(0));
    /// ```
    #[inline]
    pub fn shallow_clone(&self) -> Self {
        Self::from(self.0.lock().unwrap().clone())
    }

    /// Performs a deep structural clone of the array, returning a completely
    /// fresh copy of the same value. All elements are recursively structural
    /// cloned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Array, Value};
    /// let array = Array::new();
    /// array.push(Array::new());
    /// let clone = array.structural_clone();
    /// assert_ne!(array, clone);
    /// assert_ne!(array.get(0), clone.get(0));
    /// ```
    #[inline]
    pub fn structural_clone(&self) -> Self {
        self.0
            .lock()
            .unwrap()
            .iter()
            .map(|value| value.structural_clone())
            .collect()
    }

    /// Gets the value from a particular index in this array. Returns None
    /// if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Array, Value};
    /// let array = Array::new();
    /// array.push(3);
    /// assert_eq!(array.get(0), Some(Value::from(3)));
    /// assert_eq!(array.get(1), None);
    /// ```
    #[inline]
    pub fn get(&self, index: usize) -> Option<Value> {
        self.0.lock().unwrap().get(index).cloned()
    }

    /// Gets the value at a particular index in this array. An existing value
    /// is replaced. If the index is outside the current bounds of the array,
    /// it will be extended and filled with `unit`s as necessary.
    ///
    /// Note that since Trilogy arrays are internally mutable, you do not
    /// need a mutable reference to an Array to modify its contents.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Array, Value};
    /// let array = Array::new();
    /// array.set(3, "hello");
    /// assert_eq!(array.get(0), Some(Value::Unit));
    /// assert_eq!(array.get(3), Some(Value::from("hello")));
    /// ```
    #[inline]
    pub fn set<V: Into<Value>>(&self, index: usize, value: V) {
        let mut array = self.0.lock().unwrap();
        if array.len() <= index {
            array.resize(index + 1, Value::Unit);
        }
        array[index] = value.into();
    }

    /// Returns true of this Array contains a given value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Array, Value};
    /// let array = Array::from(vec![
    ///     Value::from(1),
    ///     Value::from(2),
    /// ]);
    /// assert!(array.contains(&Value::from(1)));
    /// assert!(!array.contains(&Value::from(4)));
    /// ```
    #[inline]
    pub fn contains(&self, value: &Value) -> bool {
        self.0.lock().unwrap().contains(value)
    }

    /// Removes the value at a particular index in this Array. Returns the removed
    /// value, or None if the index was out of bounds.
    ///
    /// If a value was removed, the length of the array will be decreased by 1.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Array, Value};
    /// let array = Array::from(vec![
    ///     Value::from(1),
    ///     Value::from(2),
    /// ]);
    /// assert_eq!(array.remove(1), Some(Value::from(2)));
    /// assert_eq!(array.len(), 1);
    /// ```
    #[inline]
    pub fn remove(&self, index: usize) -> Option<Value> {
        let mut array = self.0.lock().unwrap();
        if index <= array.len() {
            Some(array.remove(index))
        } else {
            None
        }
    }

    /// Appends an element to the back of this array.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Array, Value};
    /// let array = Array::new();
    /// array.push(3);
    /// assert_eq!(array.get(0), Some(Value::from(3)));
    /// ```
    #[inline]
    pub fn push<V: Into<Value>>(&self, value: V) {
        self.0.lock().unwrap().push(value.into());
    }

    /// Copies each element from other onto the end of self. The other array
    /// and its values are unmodified.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Array, Value};
    /// let first = Array::from(vec![Value::from(1)]);
    /// let second = Array::from(vec![Value::from(2)]);
    /// first.append(&second);
    /// assert_eq!(first.len(), 2);
    /// assert_eq!(first.get(1), Some(Value::from(2)));
    /// assert_eq!(second.len(), 1);
    /// ```
    #[inline]
    pub fn append(&self, other: &Array) {
        let mut other = other.0.lock().unwrap().clone();
        self.0.lock().unwrap().append(&mut other);
    }

    /// Returns the number of elements in the Array.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Array, Value};
    /// let array = Array::from(vec![
    ///     Value::from(1),
    ///     Value::from(2),
    /// ]);
    /// assert_eq!(array.len(), 2);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.0.lock().unwrap().len()
    }

    /// Returns true if the Array contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Array, Value};
    /// let array = Array::new();
    /// assert!(array.is_empty());
    ///
    /// array.push(1);
    /// assert!(!array.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.lock().unwrap().is_empty()
    }

    #[inline]
    pub fn range<I>(&self, range: I) -> Self
    where
        Vec<Value>: std::ops::Index<I, Output = [Value]>,
    {
        self.0.lock().unwrap()[range].to_vec().into()
    }

    /// Converts this Array to a native Rust `Vec`.
    ///
    /// Note that as the returned Vec is not an Array instance, modifications made to
    /// it will not be reflected on the underlying value. Values in this `Vec` however
    /// may still be Trilogy instances.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::runtime::{Array, Value};
    /// let array = Array::from(vec![
    ///     Value::from(1),
    ///     Value::from(2),
    /// ]);
    /// assert_eq!(array.to_vec(), vec![
    ///     Value::from(1),
    ///     Value::from(2),
    /// ]);
    /// ```
    #[inline]
    pub fn to_vec(self) -> Vec<Value> {
        self.0.lock().unwrap().clone()
    }
}

impl Display for Array {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for item in &**self.0.lock().unwrap() {
            write!(f, "{item},")?;
        }
        write!(f, "]")
    }
}

impl From<Vec<Value>> for Array {
    fn from(value: Vec<Value>) -> Self {
        Self(RefCount::new(Mutex::new(inner::ArrayInner::new(value))))
    }
}

impl From<Array> for Vec<Value> {
    fn from(value: Array) -> Self {
        (*value.0.lock().unwrap()).clone()
    }
}

impl FromIterator<Value> for Array {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        Self::from(Vec::from_iter(iter))
    }
}
