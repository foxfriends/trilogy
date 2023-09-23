/// Represents Trilogy's referential equality operator `===`
pub trait ReferentialEq {
    /// Returns true if the two values are referentially equal, or false otherwise.
    ///
    /// ```
    /// # use trilogy_vm::{Value, ReferentialEq};
    /// let first = Value::from(Vec::<Value>::new());
    /// let second = Value::from(Vec::<Value>::new());
    /// assert!(ReferentialEq::eq(&first, &first));
    /// assert!(!ReferentialEq::eq(&first, &second));
    /// ```
    fn eq(&self, other: &Self) -> bool;
}

/// Represents Trilogy's structural equality operator `==`
pub trait StructuralEq {
    /// Returns true if the two values are structurally equal, or false otherwise.
    ///
    /// ```
    /// # use trilogy_vm::{Value, StructuralEq};
    /// let first = Value::from(vec![Value::from(3)]);
    /// let second = Value::from(vec![Value::from(3)]);
    /// assert!(StructuralEq::eq(&first, &second));
    /// ```
    fn eq(&self, other: &Self) -> bool;
}
