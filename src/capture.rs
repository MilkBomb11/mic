use std::collections::{HashMap, HashSet};

use crate::ir::{Instr, Operand};

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

fn build_capture_sets_and_call_graph (instrs: &[Instr]) -> (HashMap<String, HashSet<String>>, HashMap<String, HashSet<String>>) {
    let mut call_graph: HashMap<String, HashSet<String>> = HashMap::new();
    let mut capture_sets: HashMap<String, HashSet<String>> = HashMap::new();
    for instr in instrs {
        if let Instr::FnDecl { name, params, body } = instr {
            let (captures, calls) = analyze_fn(params, body);
            call_graph.insert(name.to_owned(), calls);
            capture_sets.insert(name.to_owned(), captures);
        }
    }
    (capture_sets, call_graph)
}

fn fixed_point (instrs: &[Instr]) -> HashMap<String, HashSet<String>> {
    let (mut capture_sets, call_graph) = build_capture_sets_and_call_graph(instrs);
    let mut change = true;
    let empty: HashSet<String> = HashSet::new();
    while change {
        change = false;
        let old_capture_sets = capture_sets.clone();
        for (name, set) in capture_sets.iter_mut() {
            let calls = call_graph.get(name.as_str()).unwrap_or(&empty);
            for callee_name in calls.iter() {
                let callee_captures = old_capture_sets.get(callee_name).unwrap();
                for cap in callee_captures.iter() {
                    if !old_capture_sets.get(name).unwrap().contains(cap) {
                        set.insert(cap.to_owned());
                        change = true;
                    }
                }
            }
        }
    }
    capture_sets
}

pub fn capture (instrs: &mut [Instr]) -> () {
    let capture_sets = fixed_point(instrs);
    for instr in instrs.iter_mut() {
        if let Instr::Call { name, args, .. } = instr {
            let captures = capture_sets.get(name.as_str()).unwrap();
            for cap in captures.iter() {
                args.push(Operand::Reg(cap.clone()));
            }
        }

        if let Instr::FnDecl { name, params, body } = instr {
            let captures = capture_sets.get(name.as_str()).unwrap();
            for cap in captures.iter() {
                params.push(cap.clone());
            }
            
            for instr in body.iter_mut() {
                if let Instr::Call { name, args, .. } = instr {
                    let captures = capture_sets.get(name.as_str()).unwrap();
                    for cap in captures.iter() {
                        args.push(Operand::Reg(cap.clone()));
                    }
                }
            }
        }
    }
}