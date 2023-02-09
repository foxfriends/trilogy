use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct RecordComprehension {
    start: Token,
    pub key_expression: Expression,
    pub value_expression: Expression,
    pub query: Query,
    end: Token,
}
