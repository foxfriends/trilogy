use std::io::{self, Write};

pub fn disassemble<W: Write>(output: &mut W, program: Vec<u8>) -> io::Result<()> {
    for instruction in program.into_iter() {
        writeln!(output, "{instruction}")?;
    }
    Ok(())
}
