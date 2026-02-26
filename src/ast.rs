use crate::typ::Type;
use crate::value::Val;

#[derive(Debug)]
pub enum Expr {
    Const(usize, Val),
    Var(usize, String),
    UnOp(usize, UnaryOperator, Box<Expr>),
    BinOp(usize, Box<Expr>, BinaryOperator, Box<Expr>),
    Deref(usize, Box<Expr>),
    AddrOf(usize, Box<Expr>),
    Subscr(usize, Box<Expr>, Box<Expr>),
    Cast(usize, Box<Expr>, Type),
    Call {loc:usize, name:String, args:Vec<Box<Expr>>, ret_type:Option<Type>},
}

#[derive(Debug)]
pub enum Stmt {
    Expr(usize, Box<Expr>),
    Declare(usize, Type, String),
    Define(usize, Type, String, Box<Expr>),
    PtrUpdate(usize, Box<Expr>, Box<Expr>),
    ArrUpdate(usize, Box<Expr>, Box<Expr>, Box<Expr>),
    Assign(usize, String, Box<Expr>),
    Block(usize, Vec<Box<Stmt>>),
    If {loc:usize, cond:Box<Expr>, true_arm:Box<Stmt>, false_arm:Box<Stmt>},
    While {loc:usize, cond:Box<Expr>, body:Box<Stmt>},
    FnDecl {loc:usize, name:String, args:Vec<(String,Type)>, ret_type:Type, body:Box<Stmt>},
    Return(usize, Box<Expr>),
    Break(usize),
    Continue(usize),
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
