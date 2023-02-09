use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct NotUnification {
    start: Token,
    pub query: Unification,
}
