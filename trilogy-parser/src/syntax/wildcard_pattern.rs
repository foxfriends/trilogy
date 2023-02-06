use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned)]
pub struct WildcardPattern {
    token: Token,
}
