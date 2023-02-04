use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct BooleanUnification {
    start: Token,
    pub expression: Expression,
}
