use super::*;

#[derive(Clone, Debug, Spanned)]
pub enum Handler {
    Given(Box<GivenHandler>),
    When(Box<WhenHandler>),
}
