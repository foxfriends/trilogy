use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct Call {
    pub path: Path,
    pub arguments: Vec<Expression>,
    end: Token,
}
