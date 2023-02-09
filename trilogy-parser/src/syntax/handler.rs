use super::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum Handler {
    Given(Box<GivenHandler>),
    When(Box<WhenHandler>),
}
