use trilogy_scanner::Token;

/// Not a real AST node, but a stand-in when a section of the code fails
/// to parse. Nodes which support recovery provide a case to hold syntax
/// errors.
#[derive(Clone, Debug)]
pub struct SyntaxError {
    tokens: Vec<Token>,
    message: String,
}

impl SyntaxError {
    pub(crate) fn new(tokens: Vec<Token>, message: String) -> Self {
        Self { tokens, message }
    }

    pub(crate) fn new_spanless(message: String) -> Self {
        Self {
            tokens: vec![],
            message,
        }
    }
}
