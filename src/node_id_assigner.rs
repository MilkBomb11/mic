use crate::{ast::{Expr, Stmt}};

pub struct IdBuilder {
    node_id: usize,
}

impl IdBuilder {
    pub fn new() -> Self { 
        Self { node_id: 0 }
    }

    fn new_id(&mut self) -> usize {
        let res = self.node_id;
        self.node_id += 1;
        res
    }
}

fn assign_id_expr (expr:&mut Box<Expr>, id_builder: &mut IdBuilder) {
    match expr.as_mut() {
        Expr::Const { id, loc:_, val:_ }
        | Expr::Var { id, loc:_, name:_ } => {
            *id = id_builder.new_id();
        },
        Expr::Deref { id, loc:_, operand }
        | Expr::AddrOf { id, loc:_, operand }
        | Expr::UnOp { id, loc:_, operator:_, operand } => {
            *id = id_builder.new_id();
            assign_id_expr(operand, id_builder);
        },
        Expr::BinOp { id, loc:_, left, operator:_, right } => {
            *id = id_builder.new_id();
            assign_id_expr(left, id_builder);
            assign_id_expr(right, id_builder);
        },
        Expr::Subscr { id, loc:_, operand, index } => {
            *id = id_builder.new_id();
            assign_id_expr(operand, id_builder);
            assign_id_expr(index, id_builder);
        },
        Expr::Cast { id, loc:_, operand, typ:_ } => {
            *id = id_builder.new_id();
            assign_id_expr(operand, id_builder);
        },
        Expr::Call { id, loc:_, name:_, args, ret_type:_ } => {
            *id = id_builder.new_id();
            for arg in args.iter_mut() {
                assign_id_expr(arg, id_builder);
            }
        }
    }
}

fn assign_id_stmt (stmt:&mut Box<Stmt>, id_builder: &mut IdBuilder) {
    match stmt.as_mut() {
        Stmt::Break { id, loc:_ }
        | Stmt::Continue { id, loc:_ }
        | Stmt::Declare {id, loc:_, typ:_, name:_} => {
            *id = id_builder.new_id();
        },
        Stmt::Return { id, loc:_, expr } 
        | Stmt::Expr { id, loc:_, expr }=> {
            *id = id_builder.new_id();
            assign_id_expr(expr, id_builder);
        },
        Stmt::Assign { id, loc:_, lhs, rhs } => {
            *id = id_builder.new_id();
            assign_id_expr(rhs, id_builder);
            assign_id_expr(lhs, id_builder);
        },
        Stmt::Define { id, loc:_, typ:_, name:_, rhs } => {
            *id = id_builder.new_id();
            assign_id_expr(rhs, id_builder);
        },
        Stmt::If { id, loc:_, cond, true_arm, false_arm } => {
            *id = id_builder.new_id();
            assign_id_expr(cond, id_builder);
            assign_id_stmt(true_arm, id_builder);
            assign_id_stmt(false_arm, id_builder);
        },
        Stmt::While { id, loc:_, cond, body } => {
            *id = id_builder.new_id();
            assign_id_expr(cond, id_builder);
            assign_id_stmt(body, id_builder);
        },
        Stmt::FnDecl { id, loc:_, name:_, args:_, ret_type:_, body } => {
            *id = id_builder.new_id();
            assign_id_stmt(body, id_builder);
        },
        Stmt::Block { id, loc:_, stmts } => {
            *id = id_builder.new_id();
            for stmt in stmts.iter_mut() {
                assign_id_stmt(stmt, id_builder);
            }
        }
    }
}

pub fn assign_id (stmts:&mut Vec<Box<Stmt>>, id_builder: &mut IdBuilder) {
    for stmt in stmts.iter_mut() {
        assign_id_stmt(stmt, id_builder);
    }
}