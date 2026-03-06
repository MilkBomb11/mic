use std::collections::{HashMap, HashSet};

use crate::ir::{Instr, Operand};
type Set = HashSet<String>;
type Map = HashMap<String, Set>;

fn analyze_fn(params: &[String], body: &[Instr]) -> (Set, Set, Set) {
    let mut defined: HashSet<String> = params.iter().cloned().collect();
    let mut captures: HashSet<String> = HashSet::new();
    let mut calls: HashSet<String> = HashSet::new();

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
    (captures, calls, defined)
}

fn build_capture_sets_and_call_graph (instrs: &[Instr]) -> (Map, Map, Map) {
    let mut call_graph: Map = HashMap::new();
    let mut capture_sets: Map = HashMap::new();
    let mut defined_sets: Map = HashMap::new();
    for instr in instrs {
        if let Instr::FnDecl { name, params, body } = instr {
            let (captures, calls, defined) = analyze_fn(params, body);
            call_graph.insert(name.to_owned(), calls);
            capture_sets.insert(name.to_owned(), captures);
            defined_sets.insert(name.to_owned(), defined);
        }
    }
    (capture_sets, call_graph, defined_sets)
}

fn fixed_point (instrs: &[Instr]) -> Map {
    let (mut capture_sets, call_graph, defined_sets) = build_capture_sets_and_call_graph(instrs);
    let mut change = true;
    let empty: HashSet<String> = HashSet::new();
    while change {
        change = false;
        let old_capture_sets = capture_sets.clone();
        for (name, set) in capture_sets.iter_mut() {
            let calls = call_graph.get(name.as_str()).unwrap_or(&empty);
            let caller_defined = defined_sets.get(name.as_str()).unwrap_or(&empty);
            for callee_name in calls.iter() {
                let callee_captures = old_capture_sets.get(callee_name).unwrap_or(&empty);
                for cap in callee_captures.iter() {
                    if !caller_defined.contains(cap) && !old_capture_sets.get(name).unwrap().contains(cap) {
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

    let get_sorted_caps = |name: &str| -> Vec<String> {
        if let Some(caps) = capture_sets.get(name) {
            let mut sorted: Vec<String> = caps.iter().cloned().collect();
            sorted.sort();
            sorted
        }
        else { Vec::new() }
    };

    for instr in instrs.iter_mut() {
        if let Instr::Call { name, args, .. } = instr {
            for cap in get_sorted_caps(name.as_str()) {
                args.push(Operand::Reg(cap.clone()));
            }
        }

        if let Instr::FnDecl { name, params, body } = instr {
            for cap in get_sorted_caps(name.as_str()) {
                params.push(cap.clone());
            }
            
            for instr in body.iter_mut() {
                if let Instr::Call { name, args, .. } = instr {
                    for cap in get_sorted_caps(name.as_str()) {
                        args.push(Operand::Reg(cap.clone()));
                    }
                }
            }
        }
    }
}