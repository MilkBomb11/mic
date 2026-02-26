use crate::{ast::{BinaryOperator, Expr, UnaryOperator}, symbol_table::SymbolTable, typ::Type, value::Val};

#[derive(Debug, Clone)]
struct Error {
    loc: usize,
    msg: String
}


fn type_check_expr (expr:& mut Box<Expr>, sym_tab:&SymbolTable<Type>) -> Result<(Type, bool), Error> {
    match expr.as_mut() {
        Expr::Const(_, Val::Int(_)) => Ok((Type::Int, false)),
        Expr::Const(_, Val::Byte(_)) => Ok((Type::Byte, false)),
        Expr::Const(_, Val::Bool(_)) => Ok((Type::Bool, false)),
        Expr::Var(loc, x) => {
            if let Some(t) = sym_tab.lookup(x.as_str())  { Ok((t.to_owned(), true)) }
            else { Err(Error { 
                    loc: *loc, 
                    msg: format!("Use of undeclared variable {}.", x)
                })
            }
        },
        Expr::AddrOf(loc, expr) => {
            match type_check_expr(expr, sym_tab) {
                Err(err) => Err(err),
                Ok((t, true)) => Ok((Type::Ptr(Box::new(t)), false)),
                Ok((t, false)) => Err(Error { 
                    loc: *loc,
                    msg: format!("Cannot get lvalue of type {:?}", t) 
                }),
            }
        },
        Expr::Deref(loc, expr) => {
            match type_check_expr(expr, sym_tab) {
                Err(err) => Err(err),
                Ok((Type::Ptr(t), _)) => Ok((t.as_ref().to_owned(), true)),
                Ok((Type::Arr(t, _), _)) => Ok((t.as_ref().to_owned(), true)),
                Ok((t, _)) => Err(Error {
                    loc: *loc,
                    msg: format!("Cannot dereference value of type {:?}", t)
                }),
            }
        },
        Expr::Subscr(loc, left , right) => {
            match (type_check_expr(left, sym_tab), type_check_expr(right, sym_tab)) {
                (Err(err), _) => Err(err),
                (_, Err(err)) => Err(err),
                (Ok((Type::Ptr(t), _)), Ok((Type::Int, _))) => Ok((t.as_ref().to_owned(), true)),
                (Ok((Type::Arr(t, _), _)), Ok((Type::Int, _))) => Ok((t.as_ref().to_owned(), true)),
                (Ok((t1, _)), Ok((t2, _))) => Err(Error { 
                    loc: *loc, 
                    msg: format!("Cannot perform {:?}[{:?}]", t1, t2) 
                })
            }
        },
        Expr::Cast(loc, expr, typ) => {
            match type_check_expr(expr, sym_tab) {
                Err(err) => Err(err),
                Ok((Type::Func {name, args:_, ret_type:_}, _)) => Err(Error {
                    loc: *loc,
                    msg: format!("Cannot cast function {}", name)
                }),
                Ok((_, _)) => Ok((typ.to_owned(), false))
            }
        },
        Expr::BinOp(loc, left, op, right) => {
            let left_type = 
                match type_check_expr(left, sym_tab) {
                    Ok((t, _)) => t,
                    Err(err) => return Err(err),
                };
            let right_type = 
                match type_check_expr(right, sym_tab) {
                    Ok((t, _)) => t,
                    Err(err) => return Err(err),
                };
            match op {
                BinaryOperator::Add => {
                    match (left_type.clone(), right_type.clone()) {
                        (Type::Arr(t,n), Type::Int) => Ok((Type::Arr(t, n), false)),
                        (Type::Int, Type::Arr(t,n)) => Ok((Type::Arr(t, n), false)),
                        (Type::Ptr(t), Type::Int) => Ok((Type::Ptr(t), false)),
                        (Type::Int, Type::Ptr(t)) => Ok((Type::Ptr(t), false)),
                        (Type::Int, Type::Int) => Ok((Type::Int, false)),
                        _ => Err(Error { 
                            loc: *loc, 
                            msg: format!("Cannot perform operation {:?} + {:?}", left_type, right_type) })
                    }
                },
                BinaryOperator::Sub => {
                    match (left_type.clone(), right_type.clone()) {
                        (Type::Ptr(t1), Type::Ptr(t2)) => {
                            if t1 == t2 { Ok((Type::Int, false)) }
                            else {Err(Error { 
                                loc: *loc, 
                                msg: format!("Cannot perform operation {:?} - {:?}", left_type, right_type) 
                                })
                            }
                        },
                        (Type::Ptr(t1), Type::Int) => { Ok((Type::Ptr(t1), false)) }
                        (Type::Int, Type::Int) => Ok((Type::Int, false)),
                        (t1, t2) => { Err(Error { 
                                loc: *loc, 
                                msg: format!("Cannot perform operation {:?} - {:?}", t1, t2)
                            }) 
                        }
                    }
                },
                BinaryOperator::Mul | BinaryOperator::Div => {
                    if left_type == Type::Int && right_type == Type::Int { Ok((Type::Int, false)) }
                    else {
                        Err(Error { loc: *loc, msg: format!("Cannot perform * or / on {:?}, {:?}", left_type, right_type) })
                    }
                },
                BinaryOperator::Eq | BinaryOperator::Neq => {
                    if left_type == right_type {Ok((Type::Bool, false))}
                    else {
                        Err(Error { 
                            loc: *loc, 
                            msg: format!("Cannot perform == or != on {:?}, {:?}", left_type, right_type) 
                        })
                    }
                },
                BinaryOperator::Lt | BinaryOperator::Leq 
                | BinaryOperator::Gt | BinaryOperator::Geq => {
                    if left_type == right_type {Ok((Type::Bool, false))}
                    else {
                        Err(Error { 
                            loc: *loc, 
                            msg: format!("Cannot perform <, <=, >, > on {:?}, {:?}", left_type, right_type) 
                        })
                    }
                },
                BinaryOperator::And | BinaryOperator::Or => {
                    if left_type == Type::Bool && right_type == Type::Bool { Ok((Type::Bool, false)) }
                    else {
                        Err(Error { 
                            loc: *loc, 
                            msg: format!("Cannot perform &&, || on {:?}, {:?}", left_type, right_type) 
                        })
                    }
                }
            }
        },
        Expr::UnOp(loc, op, expr) => {
            let expr_type = 
                match type_check_expr(expr, sym_tab) {
                    Ok((t, _)) => t,
                    Err(e) => return Err(e),
                };
            match op {
                UnaryOperator::Pos |
                UnaryOperator::Neg => {
                    if expr_type == Type::Int { Ok((Type::Int, false)) }
                    else { Err(Error {
                            loc: *loc,
                            msg: format!("Cannot perform - or +{:?}", expr_type),
                        })
                    } 
                },
                UnaryOperator::Not => {
                    if expr_type == Type::Bool { Ok((Type::Bool, false)) }
                    else { Err(Error {
                            loc: *loc,
                            msg: format!("Cannot perform !{:?}", expr_type),
                        })
                    }
                }
            }
        },
        Expr::Call { loc, name, args, ret_type} => {
            let mut call_arg_types: Vec<Type> = Vec::new();
            for arg in args.iter_mut() {
                match type_check_expr(arg, sym_tab) {
                    Ok((t, _)) => call_arg_types.push(t),
                    Err(err) => return Err(err)
                }
            }

            let f_type =
                match sym_tab.lookup(name.as_str()) {
                    None => return Err(Error { 
                        loc: *loc, 
                        msg: format!("Undefined function {}", name) 
                    }),
                    Some(t) => t.to_owned()
                };
            match f_type {
                Type::Func { name:_, args:f_args, ret_type:f_ret_type} => {
                    if call_arg_types.len() != f_args.len() { 
                        return Err(Error { 
                            loc: *loc, 
                            msg: format!("Argument length mismatch of {}.", name) 
                    }) }
                    for (i, formal_arg) in f_args.iter().enumerate() {
                        if &call_arg_types[i] != formal_arg.as_ref() {
                            return Err(Error {
                                loc: *loc,
                                msg: format!("Type mismatch of {} on argument {}. Expected {:?} but got {:?}.", name, i, formal_arg, call_arg_types[i])
                            })
                        }
                    }
                    let resolved_type = f_ret_type.as_ref().to_owned();
                    *ret_type = Some(resolved_type.clone());
                    Ok((resolved_type, false))
                }
                t => Err(Error { 
                    loc: *loc, 
                    msg: format!("Tried to call a {:?}", t) 
                })
            }
        }
    }
}