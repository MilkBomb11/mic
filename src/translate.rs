use std::{collections::HashMap};

use crate::{Error, ast::{BinaryOperator, Expr, Stmt, UnaryOperator}, ir::{BinaryOp, IRBuilder, Instr, Lbl, Operand, Size, UnaryOp}};
use crate::{jump_context::{JumpContext}, symbol_table::SymbolTable, typ::Type, value::Val};

fn size_type_of_type(t:&Type) -> Size {
    match t {
        Type::Arr(_, _) 
        | Type::Ptr(_) => Size::Long,
        Type::Bool
        | Type::Byte => Size::Byte,
        Type::Int => Size::Word,
        Type::Func { .. } => panic!("Cannot get size of type {:?}", t)
    }
}

fn size_of_type(t:&Type) -> i32 {
    match t {
        Type::Arr(t, n) => (*n as i32) * size_of_type(t.as_ref()),
        Type::Ptr(_) => 8,
        Type::Int => 4,
        Type::Bool | Type::Byte => 1,
        Type::Func { .. } => panic!("Cannot get size of type {:?}", t)
    }
}

fn val_as_i32(v:&Val) -> i32 {
    match v {
        Val::Bool(v) => v.to_owned() as i32,
        Val::Byte(v) => v.to_owned() as i32,
        Val::Int(v) => v.to_owned(),
    }
}

fn binop_ast_to_ir (operator:&BinaryOperator) -> BinaryOp {
    match operator {
        BinaryOperator::Add => BinaryOp::Add,
        BinaryOperator::Sub => BinaryOp::Sub,
        BinaryOperator::Mul => BinaryOp::Mul,
        BinaryOperator::Div => BinaryOp::Div,
        BinaryOperator::Lt => BinaryOp::Lt,
        BinaryOperator::Leq => BinaryOp::Leq,
        BinaryOperator::Gt => BinaryOp::Gt,
        BinaryOperator::Geq => BinaryOp::Geq,
        BinaryOperator::Eq => BinaryOp::Eq,
        BinaryOperator::Neq => BinaryOp::Neq,
        BinaryOperator::And 
        | BinaryOperator::Or => panic!("operator {:?} has no equivalent ir counterpart.", operator)
    }
}

fn unop_ast_to_ir (operator:&UnaryOperator) -> UnaryOp {
    match operator {
        UnaryOperator::Neg => UnaryOp::Neg,
        UnaryOperator::Not => UnaryOp::Not,
        UnaryOperator::Pos => UnaryOp::Pos,
    }
}

fn translate_rvalue(expr:&Box<Expr>, ir_builder:&mut IRBuilder, sym_tab:&SymbolTable<String>, ntm:&HashMap<usize, Type>) -> String {
    match expr.as_ref() {
        Expr::Var { id, .. }
        | Expr::Deref { id, .. }
        | Expr::Subscr { id, .. } => {
            let expr_type = ntm.get(id).unwrap();
            let addr_reg: String = translate_lvalue(expr, ir_builder, sym_tab, ntm);
            if let Type::Arr(_, _) = expr_type { return addr_reg; }
            let val_reg = ir_builder.new_reg();
            ir_builder.emit(Instr::Load { size: size_type_of_type(expr_type), dest: val_reg.clone(), src: addr_reg });
            val_reg
        },
        Expr::Const { id:_, loc:_, val } => {
            let r1 = ir_builder.new_reg();
            ir_builder.emit(Instr::Set { dest: r1.clone(), src: Operand::Imm(val_as_i32(val))});
            r1
        },
        Expr::AddrOf { id:_, loc:_, operand } => translate_lvalue(operand, ir_builder, sym_tab, ntm),
        Expr::BinOp { id:_, loc:_, left, operator, right } => {
            match operator {
                BinaryOperator::Add => {
                    let r1 = translate_rvalue(left, ir_builder, sym_tab, ntm);
                    let r2 = translate_rvalue(right, ir_builder, sym_tab, ntm);
                    let left_type = ntm.get(&left.id()).unwrap();
                    let right_type = ntm.get(&right.id()).unwrap();
                    match (left_type, right_type) {
                        (Type::Arr(inner, _), Type::Int)
                        | (Type::Ptr(inner), Type::Int) => {
                            let factor = size_of_type(inner.as_ref());
                            let sized_r2 = ir_builder.new_reg();
                            let r3 = ir_builder.new_reg();
                            ir_builder.emit(Instr::BinOp { 
                                dest: sized_r2.clone(), 
                                left: Operand::Reg(r2), 
                                operator: BinaryOp::Mul, 
                                right: Operand::Imm(factor) });
                            ir_builder.emit(Instr::BinOp { 
                                dest: r3.clone(), 
                                left: Operand::Reg(r1), 
                                operator: BinaryOp::Add, 
                                right: Operand::Reg(sized_r2) });
                            r3
                        },
                        (Type::Int, Type::Arr(inner, _))
                        | (Type::Int, Type::Ptr(inner)) => {
                            let factor = size_of_type(inner.as_ref());
                            let sized_r1 = ir_builder.new_reg();
                            let r3 = ir_builder.new_reg();
                            ir_builder.emit(Instr::BinOp { 
                                dest: sized_r1.clone(), 
                                left: Operand::Reg(r1), 
                                operator: BinaryOp::Mul, 
                                right: Operand::Imm(factor) });
                            ir_builder.emit(Instr::BinOp { 
                                dest: r3.clone(), 
                                left: Operand::Reg(r2), 
                                operator: BinaryOp::Add, 
                                right: Operand::Reg(sized_r1) });
                            r3
                        },
                        _ => {
                            let r3 = ir_builder.new_reg();
                            ir_builder.emit(Instr::BinOp { 
                                dest: r3.clone(), 
                                left: Operand::Reg(r1), 
                                operator: binop_ast_to_ir(operator), 
                                right: Operand::Reg(r2) });
                            r3
                        }
                    }
                }
                BinaryOperator::Sub => {
                    let r1 = translate_rvalue(left, ir_builder, sym_tab, ntm);
                    let r2 = translate_rvalue(right, ir_builder, sym_tab, ntm);
                    let left_type = ntm.get(&left.id()).unwrap();
                    let right_type = ntm.get(&right.id()).unwrap();
                    match (left_type, right_type) {
                        (Type::Arr(inner, _), Type::Int)
                        | (Type::Ptr(inner), Type::Int) => {
                            let factor = size_of_type(inner.as_ref());
                            let sized_r2 = ir_builder.new_reg();
                            let r3 = ir_builder.new_reg();
                            ir_builder.emit(Instr::BinOp { 
                                dest: sized_r2.clone(), 
                                left: Operand::Reg(r2), 
                                operator: BinaryOp::Mul, 
                                right: Operand::Imm(factor) });
                            ir_builder.emit(Instr::BinOp { 
                                dest: r3.clone(), 
                                left: Operand::Reg(r1),
                                operator: BinaryOp::Sub,
                                right: Operand::Reg(sized_r2) });
                            r3
                        },
                        (Type::Ptr(inner), Type::Ptr(_)) => {
                            let r3 = ir_builder.new_reg();
                            ir_builder.emit(Instr::BinOp { 
                                dest: r3.clone(), 
                                left: Operand::Reg(r1),
                                operator: BinaryOp::Sub,
                                right: Operand::Reg(r2) });
                            let factor = size_of_type(inner.as_ref());
                            let r4 = ir_builder.new_reg();
                            ir_builder.emit(Instr::BinOp { 
                                dest: r4.clone(), 
                                left:Operand::Reg(r3), operator: BinaryOp::Div, right: Operand::Imm(factor) });
                            r4
                        }
                        _ => {
                            let r3 = ir_builder.new_reg();
                            ir_builder.emit(Instr::BinOp { 
                                dest: r3.clone(), 
                                left: Operand::Reg(r1),
                                operator: BinaryOp::Sub,
                                right: Operand::Reg(r2) });
                            
                            r3
                        }
                    }
                }
                BinaryOperator::Mul
                | BinaryOperator::Div
                | BinaryOperator::Lt
                | BinaryOperator::Leq
                | BinaryOperator::Gt
                | BinaryOperator::Geq
                | BinaryOperator::Eq
                | BinaryOperator::Neq => {
                    let r1 = translate_rvalue(left, ir_builder, sym_tab, ntm);
                    let r2 = translate_rvalue(right, ir_builder, sym_tab, ntm);
                    let r3 = ir_builder.new_reg();
                    ir_builder.emit(Instr::BinOp { dest: r3.clone(), left: Operand::Reg(r1), operator: binop_ast_to_ir(operator), right: Operand::Reg(r2) });
                    r3
                }
                BinaryOperator::Or => {
                    let l_true = ir_builder.new_label();
                    let l_end = ir_builder.new_label();
                    let r3 = ir_builder.new_reg();

                    let r1 = translate_rvalue(left, ir_builder, sym_tab, ntm);
                    ir_builder.emit(Instr::GotoIf { cond: Operand::Reg(r1), dest: Lbl::Name(l_true.clone()) });

                    let r2 = translate_rvalue(right, ir_builder, sym_tab, ntm);
                    ir_builder.emit(Instr::Set { dest: r3.clone(), src: Operand::Reg(r2) });
                    ir_builder.emit(Instr::Goto { dest: Lbl::Name(l_end.clone()) });

                    ir_builder.emit(Instr::Label { label: Lbl::Name(l_true) });
                    ir_builder.emit(Instr::Set { dest: r3.clone(), src: Operand::Imm(1) });

                    ir_builder.emit(Instr::Label { label: Lbl::Name(l_end) });
                    r3
                },
                BinaryOperator::And => {
                    let l_false = ir_builder.new_label();
                    let l_end = ir_builder.new_label();
                    let r3 = ir_builder.new_reg();

                    let r1 = translate_rvalue(left, ir_builder, sym_tab, ntm);
                    ir_builder.emit(Instr::GotoIfFalse { cond: Operand::Reg(r1), dest: Lbl::Name(l_false.clone()) });

                    let r2 = translate_rvalue(right, ir_builder, sym_tab, ntm);
                    ir_builder.emit(Instr::Set { dest: r3.clone(), src: Operand::Reg(r2) });
                    ir_builder.emit(Instr::Goto { dest: Lbl::Name(l_end.clone()) });

                    ir_builder.emit(Instr::Label { label: Lbl::Name(l_false) });
                    ir_builder.emit(Instr::Set { dest: r3.clone(), src: Operand::Imm(0) });

                    ir_builder.emit(Instr::Label { label: Lbl::Name(l_end) });
                    r3
                }
            }
        }
        Expr::UnOp { id:_, loc:_, operator, operand } => {
            let r1 = translate_rvalue(operand, ir_builder, sym_tab, ntm);
            let r2 = ir_builder.new_reg();
            ir_builder.emit(Instr::UnOp { dest: r2.clone(), operator:unop_ast_to_ir(operator), operand: Operand::Reg(r1) });
            r2
        }
        Expr::Cast { id:_, loc:_, operand, typ:_ } => {
            let r1 = translate_rvalue(operand, ir_builder, sym_tab, ntm);
            r1
        }
        Expr::Call { id:_, loc:_, name, args, ret_type:_ } => {
            let regs: Vec<Operand> = 
                args
                .iter()
                .map(|arg| translate_rvalue(arg, ir_builder, sym_tab, ntm))
                .map(|reg| Operand::Reg(reg))
                .collect();
            let r_dest = ir_builder.new_reg();
            ir_builder.emit(Instr::Call { dest: r_dest.clone(), name: name.to_owned(), args: regs });
            r_dest
        }
    }
}

fn translate_lvalue(expr:&Box<Expr>, ir_builder:&mut IRBuilder, sym_tab:&SymbolTable<String>, ntm:&HashMap<usize, Type>) -> String {
    match expr.as_ref() {
        Expr::Var { id:_, loc:_, name } => sym_tab.lookup(name.as_str()).unwrap().to_owned(),
        Expr::Deref { id:_, loc:_, operand } => {
            let r1 = translate_rvalue(operand, ir_builder, sym_tab, ntm);
            r1
        }
        Expr::Subscr { id:_, loc:_, operand, index } => {
            let base_addr_reg = translate_rvalue(operand, ir_builder, sym_tab, ntm);
            let index_reg = translate_rvalue(index, ir_builder, sym_tab, ntm);
            let operand_type = ntm.get(&operand.id()).unwrap();
            match operand_type {
                Type::Arr(inner, _)
                | Type::Ptr(inner) => {
                    let factor = size_of_type(inner.as_ref());
                    let offset_reg = ir_builder.new_reg();
                    ir_builder.emit(Instr::BinOp { 
                        dest: offset_reg.clone(), 
                        left: Operand::Reg(index_reg),
                        operator: BinaryOp::Mul, 
                        right: Operand::Imm(factor) });
                    let addr_reg = ir_builder.new_reg();
                    ir_builder.emit(Instr::BinOp { 
                        dest: addr_reg.clone(), left: Operand::Reg(base_addr_reg), operator: BinaryOp::Add, right: Operand::Reg(offset_reg)});
                    addr_reg
                }
                _ => panic!("Typecheck failure: {:?}", operand_type)
            }
        }
        _ => panic!("IR generation failure: tried to translate lvalue of {:?}", expr)
    }
}

fn translate_stmt(
    stmt:&Box<Stmt>, 
    ir_builder:&mut IRBuilder, 
    sym_tab:&mut SymbolTable<String>,
    jmp_ctx:&mut JumpContext, 
    ntm:&HashMap<usize, Type>) -> Result<(), Error> {
    match stmt.as_ref() {
        Stmt::Declare { id:_, loc:_, typ, name } => {
            let addr_reg = ir_builder.new_reg();
            let size = size_of_type(typ);
            ir_builder.emit(Instr::Alloc { dest: addr_reg.clone(), size: size as usize });
            sym_tab.define(name.as_str(), addr_reg);
            Ok(())
        },
        Stmt::Define { id:_, loc:_, typ, name, rhs } => {
            let addr_reg = ir_builder.new_reg();
            let size = size_of_type(typ);
            ir_builder.emit(Instr::Alloc { dest: addr_reg.clone(), size: size as usize });
            let val_reg = translate_rvalue(rhs, ir_builder, sym_tab, ntm);
            ir_builder.emit(Instr::Store { 
                size: size_type_of_type(typ), 
                src: Operand::Reg(val_reg), 
                dest: addr_reg.clone() });
            sym_tab.define(name.as_str(), addr_reg);
            Ok(())
        },
        Stmt::Assign { id:_, loc:_, lhs, rhs } => {
            let dest_reg = translate_lvalue(lhs, ir_builder, sym_tab, ntm);
            let val_reg = translate_rvalue(rhs, ir_builder, sym_tab, ntm);
            let lhs_type = ntm.get(&lhs.id()).unwrap();
            ir_builder.emit(Instr::Store { 
                size: size_type_of_type(lhs_type), 
                src: Operand::Reg(val_reg), 
                dest: dest_reg });
            Ok(())
        },
        Stmt::If { id:_, loc:_, cond, true_arm, false_arm } => {
            let lx = ir_builder.new_label();
            let lf = ir_builder.new_label();
            let cond_reg = translate_rvalue(cond, ir_builder, sym_tab, ntm);
            ir_builder.emit(Instr::GotoIfFalse { cond: Operand::Reg(cond_reg), dest: Lbl::Name(lf.clone()) });

            sym_tab.push();
            translate_stmt(true_arm, ir_builder, sym_tab, jmp_ctx, ntm)?;
            sym_tab.pop();

            ir_builder.emit(Instr::Goto { dest: Lbl::Name(lx.clone()) });
            ir_builder.emit(Instr::Label { label: Lbl::Name(lf) });

            sym_tab.push();
            translate_stmt(false_arm, ir_builder, sym_tab, jmp_ctx, ntm)?;
            sym_tab.pop();

            ir_builder.emit(Instr::Label { label: Lbl::Name(lx) });
            Ok(())
        },
        Stmt::While { id:_, loc:_, cond, body } => {
            let lx = ir_builder.new_label();
            let lc = ir_builder.new_label();
            ir_builder.emit(Instr::Label { label: Lbl::Name(lc.clone()) });
            let cond_reg = translate_rvalue(cond, ir_builder, sym_tab, ntm);
            ir_builder.emit(Instr::GotoIfFalse { cond: Operand::Reg(cond_reg), dest: Lbl::Name(lx.clone()) });

            jmp_ctx.push(lx.as_str(), lc.as_str());
            sym_tab.push();
            translate_stmt(body, ir_builder, sym_tab, jmp_ctx, ntm)?;
            sym_tab.pop();
            jmp_ctx.pop();

            ir_builder.emit(Instr::Goto { dest: Lbl::Name(lc) });
            ir_builder.emit(Instr::Label { label: Lbl::Name(lx) });
            Ok(())
        },
        Stmt::Break { id:_, loc } => {
            if let Some(x) = jmp_ctx.lookup_break() {
                ir_builder.emit(Instr::Goto { dest: Lbl::Name(x.to_owned()) });
                return Ok(());
            }
            Err(Error { 
                loc: *loc, 
                msg: format!("Statement break outside any loop") })
        },
        Stmt::Continue { id:_, loc } => {
            if let Some(x) = jmp_ctx.lookup_continue() {
                ir_builder.emit(Instr::Goto { dest: Lbl::Name(x.to_owned()) });
                return Ok(());
            }
            Err(Error { 
                loc: *loc, 
                msg: format!("Statement continue outside any loop") })
        },
        Stmt::Block { id:_, loc:_, stmts } => {
            sym_tab.push();
            for stmt in stmts.iter() {
                translate_stmt(stmt, ir_builder, sym_tab, jmp_ctx, ntm)?;
            }
            sym_tab.pop();
            Ok(())
        },
        Stmt::Return { id:_, loc:_, expr } => {
            let r = translate_rvalue(expr, ir_builder, sym_tab, ntm);
            ir_builder.emit(Instr::Ret { operand: Operand::Reg(r) });
            Ok(())
        },
        Stmt::PrintByte { id:_, loc:_, expr } => {
            let r = translate_rvalue(expr, ir_builder, sym_tab, ntm);
            ir_builder.emit(Instr::PrintByte { operand: Operand::Reg(r) });
            Ok(())
        },
        Stmt::PrintInt { id:_, loc:_, expr } => {
            let r = translate_rvalue(expr, ir_builder, sym_tab, ntm);
            ir_builder.emit(Instr::PrintInt { operand: Operand::Reg(r) });
            Ok(())
        },
        Stmt::PrintString { id:_, loc:_, expr } => {
            let r = translate_rvalue(expr, ir_builder, sym_tab, ntm);
            ir_builder.emit(Instr::PrintString { src: Operand::Reg(r) });
            Ok(())
        },
        Stmt::Expr { id:_, loc:_, expr } => {
            translate_rvalue(expr, ir_builder, sym_tab, ntm);
            Ok(())
        },
        Stmt::FnDecl { id:_, loc:_, name, args, ret_type:_, body } => {
            let mut body_ir_builder = IRBuilder::new();
            body_ir_builder.set_counters(ir_builder.get_counters());

            let mut body_jmp_ctx = JumpContext::new();
            sym_tab.push();
            for (arg_name, arg_type) in args.iter() {
                let addr_reg = body_ir_builder.new_reg();
                body_ir_builder.emit(Instr::Alloc { dest: addr_reg.clone(), size: size_of_type(arg_type) as usize });
                body_ir_builder.emit(Instr::Store { size: size_type_of_type(arg_type), src: Operand::Reg(arg_name.to_owned()), dest: addr_reg.clone() });
                sym_tab.define(arg_name.as_str(), addr_reg);
            }
            sym_tab.push();
            translate_stmt(body, &mut body_ir_builder, sym_tab, &mut body_jmp_ctx, ntm)?;
            sym_tab.pop();
            sym_tab.pop();

            ir_builder.set_counters(body_ir_builder.get_counters());

            let params: Vec<String> = 
                args
                .iter()
                .map(|(name, _)| name.to_owned())
                .collect();
            ir_builder.emit(Instr::FnDecl { 
                name: name.to_owned(), 
                params,
                body: body_ir_builder.instrs });
            Ok(())
        }
    }
}

pub fn translate_stmts(
    stmts:&Vec<Box<Stmt>>, 
    ir_builder:&mut IRBuilder, 
    ntm:&HashMap<usize, Type>) -> Result<(), Error> {
    let mut sym_tab: SymbolTable<String> = SymbolTable::new();
    sym_tab.push();
    let mut jmp_ctx: JumpContext = JumpContext::new();
    for stmt in stmts.iter() {
        match translate_stmt(stmt, ir_builder, &mut sym_tab, &mut jmp_ctx, ntm) {
            Err(err) => return Err(err),
            Ok(()) => ()
        }
    }
    sym_tab.pop();
    Ok(())
}