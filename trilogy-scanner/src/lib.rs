//! The scanner (lexer) for the Trilogy Programming Language.

mod scanner;
mod token;
mod token_type;
mod token_value;

pub use scanner::Scanner;
pub use token::Token;
pub use token_type::TokenType;
pub use token_value::TokenValue;

#[cfg(test)]
mod test;
