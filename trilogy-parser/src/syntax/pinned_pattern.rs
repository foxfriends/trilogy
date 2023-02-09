use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct PinnedPattern {
    start: Token,
    pub identifier: Identifier,
}
