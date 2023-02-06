use super::*;

#[derive(Clone, Debug, Spanned)]
pub struct Application {
    pub function: Expression,
    pub argument: Expression,
}
