use super::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct Application {
    pub function: Expression,
    pub argument: Expression,
}
