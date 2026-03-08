use crate::{Error, ast::{Expr, Stmt}, value::Val};

fn return_check_stmt (stmt: &Box<Stmt>) -> Result<bool, Error> {
    match stmt.as_ref() {
        Stmt::Return { .. } => Ok(true),
        Stmt::Block { stmts, .. } => {
            let mut ret = false;
            for stmt in stmts.iter() {
                match return_check_stmt(stmt) {
                    Ok(t) => ret = ret || t,
                    Err(e) => return Err(e)
                }
            }
            Ok(ret)
        },
        Stmt::If { true_arm, false_arm, .. } => {
            let ret_true = return_check_stmt(true_arm)?;
            let ret_false = return_check_stmt(false_arm)?;
            Ok(ret_true && ret_false)
        },
        Stmt::While { cond, body , ..} => {
            match cond.as_ref() {
                Expr::Const { val: Val::Bool(true), .. } => return_check_stmt(body),
                _ => Ok(false)
            }
        },
        Stmt::FnDecl { loc, name, body, .. } => {
            let ret = return_check_stmt(body)?;
            if !ret {return Err(Error { 
                    loc: *loc, 
                    msg: format!("Function {} may not return.", name.as_str())
                })
            }
            Ok(false)
        },
        _ => Ok(false)
    }
}

pub fn return_check (stmts: &Vec<Box<Stmt>>) -> Result<bool, Error> {
    for stmt in stmts.iter() {
       return_check_stmt(stmt)?;
    }
    Ok(true)
}