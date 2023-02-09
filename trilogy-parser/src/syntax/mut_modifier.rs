use trilogy_scanner::Token;

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub enum MutModifier {
    Not,
    Mut(Token),
}
