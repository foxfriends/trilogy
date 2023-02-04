use super::*;

#[derive(Clone, Debug)]
pub enum Handler {
    Given(Box<GivenHandler>),
    When(Box<WhenHandler>),
}
