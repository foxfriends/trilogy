use super::super::Offset;
use super::AsmContext;
use super::ErrorKind;
use crate::Value;

pub(crate) trait FromAsmParam: Sized {
    fn from_asm_param(src: &str, ctx: &mut AsmContext) -> Result<Self, ErrorKind>;
}

impl FromAsmParam for Value {
    fn from_asm_param(src: &str, ctx: &mut AsmContext) -> Result<Self, ErrorKind> {
        Ok(ctx.parse_value(src)?)
    }
}

impl FromAsmParam for Offset {
    fn from_asm_param(src: &str, ctx: &mut AsmContext) -> Result<Self, ErrorKind> {
        ctx.parse_offset(src)
    }
}
