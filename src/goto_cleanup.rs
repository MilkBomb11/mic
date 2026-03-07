use crate::ir::{IRBuilder, Instr};

pub struct GotoCleanup {
    met_goto: bool,
}

impl GotoCleanup {
    pub fn new() -> Self {
        Self {
            met_goto: false
        }
    }

    pub fn cleanup (&mut self, instrs: &[Instr], ir_builder:&mut IRBuilder) {
        for instr in instrs.iter() {
            match instr {
                Instr::FnDecl { name, params, body } => {
                    let mut body_ir_builder = IRBuilder::new();
                    let saved_state = self.met_goto;
                    self.met_goto = false;
                    self.cleanup(body, &mut body_ir_builder);
                    self.met_goto = saved_state;
                    ir_builder.emit(Instr::FnDecl { 
                        name: name.to_owned(), 
                        params: params.clone(), 
                        body: body_ir_builder.instrs 
                    }); 
                },
                Instr::Goto { .. } => {
                    if !self.met_goto { 
                        self.met_goto = true;
                        ir_builder.emit(instr.to_owned());
                    }
                }
                instr => {
                    self.met_goto = false;
                    ir_builder.emit(instr.to_owned());
                },
            }
        }
    }
}
