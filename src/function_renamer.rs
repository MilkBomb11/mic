use crate::{ir::Instr, symbol_table::SymbolTable};

pub struct FunctionRenamer {
    id: u32
}

impl FunctionRenamer {
    pub fn new() -> Self {
        Self { id: 0 }
    }

    fn collect_fndecl(&mut self, sym_tab:&mut SymbolTable<String>, instrs:&mut Vec<Instr>) {
        for instr in instrs.iter_mut() {
            match instr {
                Instr::FnDecl { name, params:_, body:_ } => {
                    let new_name = format!("{}#{}", name, self.id);
                    self.id += 1;
                    sym_tab.define(name.as_str(), new_name.clone());
                    *name = new_name;
                },
                _ => ()
            }
        }
    }
    
    fn traverse_instrs(&mut self, sym_tab:&mut SymbolTable<String>, instrs:&mut Vec<Instr>) {
        self.collect_fndecl(sym_tab, instrs);
        for instr in instrs.iter_mut() {
            match instr {
                Instr::Call { dest:_, name, args:_ } => {
                    if let Some(new_name) = sym_tab.lookup(name.as_str()) {
                        *name = new_name.to_owned();
                    }
                },
                Instr::FnDecl { name:_, params:_, body } => {
                    sym_tab.push();
                    self.traverse_instrs(sym_tab, body);
                    sym_tab.pop();
                }
                _ => ()
            }
        }
    }
    
    pub fn traverse(&mut self, instrs:&mut Vec<Instr>) {
        let mut sym_tab: SymbolTable<String> = SymbolTable::new();
        sym_tab.push();
        self.traverse_instrs(&mut sym_tab, instrs);
        sym_tab.pop();
    }
}