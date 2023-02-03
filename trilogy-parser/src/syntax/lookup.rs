use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct Lookup {
    pub path: Path,
    pub patterns: Vec<Pattern>,
    end: Token,
}
