use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct EndStatement {
    start: Token,
}
