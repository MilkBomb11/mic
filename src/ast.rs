use crate::typ::Type;
use crate::value::Val;

#[derive(Debug)]
pub enum Expr {
    Const{id:usize, loc:usize, val:Val},
    Var{id:usize, loc:usize, name:String},
    UnOp{id:usize, loc:usize, operator:UnaryOperator, operand:Box<Expr>},
    BinOp{id:usize, loc:usize, left:Box<Expr>, operator:BinaryOperator, right:Box<Expr>},
    Deref{id:usize, loc:usize, operand:Box<Expr>},
    AddrOf{id:usize, loc:usize, operand:Box<Expr>},
    Subscr{id:usize, loc:usize, operand:Box<Expr>, index:Box<Expr>},
    Cast{id:usize, loc:usize, operand:Box<Expr>, typ:Type},
    Call {id:usize, loc:usize, name:String, args:Vec<Box<Expr>>, ret_type:Option<Type>},
}

impl Expr {
    pub fn id(&self) -> usize {
        match self {
            Expr::Const { id, .. }
            | Expr::AddrOf { id, .. }
            | Expr::BinOp { id, .. }
            | Expr::UnOp { id, .. }
            | Expr::Deref { id, .. }
            | Expr::Cast { id, .. }
            | Expr::Call { id, .. }
            | Expr::Var { id, .. }
            | Expr::Subscr { id, .. } => *id
        }
    }
}

#[derive(Debug)]
pub enum Stmt {
    Expr{id:usize, loc:usize, expr:Box<Expr>},
    Declare{id:usize, loc:usize, typ:Type, name:String},
    Define{id:usize, loc:usize, typ:Type, name:String, rhs:Box<Expr>},
    Assign{id:usize, loc:usize, lhs:Box<Expr>, rhs:Box<Expr>},
    Block{id:usize, loc:usize, stmts:Vec<Box<Stmt>>},
    If {id:usize, loc:usize, cond:Box<Expr>, true_arm:Box<Stmt>, false_arm:Box<Stmt>},
    While {id:usize, loc:usize, cond:Box<Expr>, body:Box<Stmt>},
    FnDecl {id:usize, loc:usize, name:String, args:Vec<(String,Type)>, ret_type:Type, body:Box<Stmt>},
    Return{id:usize, loc:usize, expr:Box<Expr>},
    Break{id:usize, loc:usize},
    Continue{id:usize, loc:usize},
}

#[derive(Debug)]
pub enum BinaryOperator {
    Add, Sub, Mul, Div,
    Lt, Leq, Gt, Geq,
    Eq, Neq, And, Or,
}

#[derive(Debug)]
pub enum UnaryOperator {
    Neg, Pos, Not,
}
