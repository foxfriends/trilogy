use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct PinnedPattern {
    start: Token,
    pub identifier: Identifier,
}
