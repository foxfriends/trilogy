mod context;
mod error;
mod from_asm_param;
mod value;

pub(crate) use context::AsmContext;
pub use error::{AsmError, ErrorKind, ValueError};

pub(crate) trait Asm: Sized {
    fn fmt_asm(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;
    fn parse_asm(src: &str, ctx: &mut AsmContext) -> Result<Self, ErrorKind>;
}
