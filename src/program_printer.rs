use std::fmt::Display;

use crate::ir::Instr;

pub struct ProgramPrinter<'a> (pub &'a [Instr]);

impl<'a> Display for ProgramPrinter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for instr in self.0.iter() {
            writeln!(f, "{}", instr)?;
        }
        Ok(())
    }
}