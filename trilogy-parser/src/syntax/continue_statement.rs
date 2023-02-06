use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned)]
pub struct ContinueStatement {
    token: Token,
}
