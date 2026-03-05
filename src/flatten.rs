use crate::ir::{IRBuilder, Instr};

fn hoist_functions(instrs: &[Instr], ir_builder: &mut IRBuilder) -> () {
    for instr in instrs {
        match instr {
            Instr::FnDecl { body, .. } => {
                hoist_functions(body, ir_builder);
                ir_builder.emit(instr.clone());
            },
            _ => ()
        }
    }
}

fn remove_fndecls(instrs: &[Instr], ir_builder: &mut IRBuilder) -> () {
    for instr in instrs {
        match instr {
            Instr::FnDecl { .. } => (),
            _ => ir_builder.emit(instr.clone()),
        }
    }
}

fn cleanup(instrs: &[Instr], ir_builder: &mut IRBuilder) -> () {
    for instr in instrs {
        match instr {
            Instr::FnDecl { name, params, body } => {
                let mut body_ir_builder = IRBuilder::new();
                remove_fndecls(body, &mut body_ir_builder);
                ir_builder.emit(Instr::FnDecl { 
                    name: name.to_owned(), 
                    params: params.to_owned(), 
                    body: body_ir_builder.instrs });
            }
            _ => ir_builder.emit(instr.clone()),
        }
    }
}

fn collect_global(instrs: &[Instr], ir_builder: &mut IRBuilder) -> () {
    for instr in instrs {
        if !matches!(instr, Instr::FnDecl { .. }) {
            ir_builder.emit(instr.to_owned());
        }
    }
}

pub fn flatten(instrs: &[Instr], ir_builder: &mut IRBuilder) -> () {
    collect_global(instrs, ir_builder);
    hoist_functions(instrs, ir_builder);
    let mut cleanup_ir_builder = IRBuilder::new();
    cleanup(&ir_builder.instrs, &mut cleanup_ir_builder);
    ir_builder.instrs = cleanup_ir_builder.instrs;
}