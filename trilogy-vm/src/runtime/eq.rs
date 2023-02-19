pub trait ReferentialEq {
    fn eq(&self, other: &Self) -> bool;
}

pub trait StructuralEq {
    fn eq(&self, other: &Self) -> bool;
}
