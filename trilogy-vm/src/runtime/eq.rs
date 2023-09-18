/// Represents Trilogy's referential equality operator `===`
pub trait ReferentialEq {
    /// Returns true if the two values are referentially equal, or false otherwise.
    fn eq(&self, other: &Self) -> bool;
}

/// Represents Trilogy's structural equality operator `==`
pub trait StructuralEq {
    /// Returns true if the two values are structurally equal, or false otherwise.
    fn eq(&self, other: &Self) -> bool;
}
