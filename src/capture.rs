use std::collections::{HashSet};

use crate::ir::{Instr};

fn analyze_fn(params: &[String], body: &[Instr]) -> (HashSet<String>, HashSet<String>) {
    let mut defined: HashSet<String> = params.iter().cloned().collect();
    let mut captures = HashSet::new();
    let mut calls = HashSet::new();

    for instr in body {
        if let Instr::Call { name, .. } = instr {
            calls.insert(name.clone());
        }
        
        let uses = instr.get_use();
        for reg in uses {
            if !defined.contains(reg) {
                captures.insert(reg.to_owned());
            }
        }
        if let Some(reg) = instr.get_def() {
            defined.insert(reg.to_owned());
        }
    }
    (captures, calls)
}
