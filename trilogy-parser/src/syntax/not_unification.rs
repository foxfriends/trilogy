use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct NotUnification {
    start: Token,
    pub query: Unification,
}
