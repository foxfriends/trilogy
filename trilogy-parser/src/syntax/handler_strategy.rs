use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum HandlerStrategy {
    Cancel(Token),
    Resume(Token),
    Invert(Token),
    Yield(Token),
}
