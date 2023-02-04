use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub enum MutModifier {
    Not,
    Mut(Token),
}
