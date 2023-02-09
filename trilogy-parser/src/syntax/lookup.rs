use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct Lookup {
    pub path: Path,
    pub patterns: Vec<Pattern>,
    end: Token,
}
