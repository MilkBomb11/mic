use crate::typ::Type;
use crate::value::Val;

#[derive(Debug)]
pub enum Expr {
    Const(Val),
    Var(String),
    UnOp(UnaryOperator, Box<Expr>),
    BinOp(Box<Expr>, BinaryOperator, Box<Expr>),
    Deref(Box<Expr>),
    AddrOf(Box<Expr>),
    Subscr(Box<Expr>, Box<Expr>),
    Cast(Box<Expr>, Type),
    Call {name:String, args:Vec<Box<Expr>>},
}

#[derive(Debug)]
pub enum Stmt {
    Expr(Box<Expr>),
    Declare(Type, String),
    Define(Type, String, Box<Expr>),
    PtrUpdate(Box<Expr>, Box<Expr>),
    ArrUpdate(Box<Expr>, Box<Expr>, Box<Expr>),
    Assign(String, Box<Expr>),
    Block(Vec<Box<Stmt>>),
    If {cond:Box<Expr>, true_arm:Box<Stmt>, false_arm:Box<Stmt>},
    While {cond:Box<Expr>, body:Box<Stmt>},
    FnDecl {name:String, args:Vec<(String,Type)>, ret_type:Type, body:Box<Stmt>},
    Return(Box<Expr>),
    Break,
    Continue,
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
