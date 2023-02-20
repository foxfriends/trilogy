use num::{BigRational, Complex};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Number(Complex<BigRational>);
