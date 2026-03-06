use std::collections::HashMap;

use crate::{ast::{BinaryOperator, Expr, Stmt, UnaryOperator}, symbol_table::SymbolTable, typ::Type, value::Val};
use crate::Error;

type NodeTypeMap = HashMap<usize, Type>;

fn type_check_expr (expr:&mut Box<Expr>, sym_tab:&SymbolTable<Type>, ntm: &mut NodeTypeMap) -> Result<(Type, bool), Error> {
    match expr.as_mut() {
        Expr::Const{id, loc:_, val:Val::Int(_)} => {
            ntm.insert(*id, Type::Int);
            Ok((Type::Int, false))
        },
        Expr::Const{id, loc:_, val:Val::Byte(_)} => {
            ntm.insert(*id, Type::Byte);
            Ok((Type::Byte, false))
        },
        Expr::Const{id, loc:_, val:Val::Bool(_)} => {
            ntm.insert(*id, Type::Bool);
            Ok((Type::Bool, false))
        },
        Expr::Var{id, loc, name:x} => {
            if let Some(t) = sym_tab.lookup(x.as_str())  { 
                ntm.insert(*id, t.to_owned());
                Ok((t.to_owned(), true))
            }
            else { Err(Error { 
                    loc: *loc, 
                    msg: format!("Use of undeclared variable {}.", x)
                })
            }
        },
        Expr::AddrOf{id, loc, operand:expr} => {
            match type_check_expr(expr, sym_tab, ntm) {
                Err(err) => Err(err),
                Ok((t, true)) => {
                    ntm.insert(*id, Type::Ptr(Box::new(t.clone())));
                    Ok((Type::Ptr(Box::new(t)), false))
                },
                Ok((t, false)) => Err(Error { 
                    loc: *loc,
                    msg: format!("Cannot get lvalue of type {}", t) 
                }),
            }
        },
        Expr::Deref{id,loc, operand:expr} => {
            match type_check_expr(expr, sym_tab, ntm) {
                Err(err) => Err(err),
                Ok((Type::Ptr(t), _)) => {
                    ntm.insert(*id, t.as_ref().to_owned());
                    Ok((t.as_ref().to_owned(), true))
                },
                Ok((Type::Arr(t, _), _)) => {
                    ntm.insert(*id, t.as_ref().to_owned());
                    Ok((t.as_ref().to_owned(), true))
                },
                Ok((t, _)) => Err(Error {
                    loc: *loc,
                    msg: format!("Cannot dereference value of type {}", t)
                }),
            }
        },
        Expr::Subscr{id, loc, operand:left , index:right} => {
            match (type_check_expr(left, sym_tab, ntm), type_check_expr(right, sym_tab, ntm)) {
                (Err(err), _) => Err(err),
                (_, Err(err)) => Err(err),
                (Ok((Type::Ptr(t), _)), Ok((Type::Int, _))) => {
                    ntm.insert(*id, t.as_ref().to_owned());
                    Ok((t.as_ref().to_owned(), true))
                },
                (Ok((Type::Arr(t, _), _)), Ok((Type::Int, _))) => {
                    ntm.insert(*id, t.as_ref().to_owned());
                    Ok((t.as_ref().to_owned(), true))
                },
                (Ok((t1, _)), Ok((t2, _))) => Err(Error { 
                    loc: *loc, 
                    msg: format!("Cannot perform {}[{}]", t1, t2) 
                })
            }
        },
        Expr::Cast{id, loc, operand:expr, typ} => {
            match type_check_expr(expr, sym_tab, ntm) {
                Err(err) => Err(err),
                Ok((Type::Func {name, args:_, ret_type:_}, _)) => Err(Error {
                    loc: *loc,
                    msg: format!("Cannot cast function {}", name)
                }),
                Ok((_, _)) => {
                    ntm.insert(*id, typ.to_owned());
                    Ok((typ.to_owned(), false))
                }
            }
        },
        Expr::BinOp {id, loc, left, operator:op, right} => {
            let left_type = 
                match type_check_expr(left, sym_tab, ntm) {
                    Ok((t, _)) => t,
                    Err(err) => return Err(err),
                };
            let right_type = 
                match type_check_expr(right, sym_tab, ntm) {
                    Ok((t, _)) => t,
                    Err(err) => return Err(err),
                };
            match op {
                BinaryOperator::Add => {
                    match (left_type.clone(), right_type.clone()) {
                        (Type::Arr(t,n), Type::Int) => {
                            ntm.insert(*id, Type::Arr(t.clone(), n));
                            Ok((Type::Arr(t, n), false))
                        },
                        (Type::Int, Type::Arr(t,n)) => {
                            ntm.insert(*id, Type::Arr(t.clone(), n));
                            Ok((Type::Arr(t, n), false))
                        },
                        (Type::Ptr(t), Type::Int) => {
                            ntm.insert(*id, Type::Ptr(t.clone()));
                            Ok((Type::Ptr(t), false))
                        },
                        (Type::Int, Type::Ptr(t)) => {
                            ntm.insert(*id, Type::Ptr(t.clone()));
                            Ok((Type::Ptr(t), false))
                        },
                        (Type::Int, Type::Int) => {
                            ntm.insert(*id, Type::Int);
                            Ok((Type::Int, false))
                        },
                        _ => Err(Error { 
                            loc: *loc, 
                            msg: format!("Cannot perform operation {} + {}", left_type, right_type) })
                    }
                },
                BinaryOperator::Sub => {
                    match (left_type.clone(), right_type.clone()) {
                        (Type::Ptr(t1), Type::Ptr(t2)) => {
                            if t1 == t2 { 
                                ntm.insert(*id, Type::Int);
                                Ok((Type::Int, false))
                            }
                            else {Err(Error { 
                                loc: *loc, 
                                msg: format!("Cannot perform operation {} - {}", left_type, right_type) 
                                })
                            }
                        },
                        (Type::Ptr(t1), Type::Int) => {
                            ntm.insert(*id, Type::Ptr(t1.clone()));
                            Ok((Type::Ptr(t1), false)) 
                        }
                        (Type::Int, Type::Int) => {
                            ntm.insert(*id, Type::Int);
                            Ok((Type::Int, false))
                        },
                        (t1, t2) => { Err(Error { 
                                loc: *loc, 
                                msg: format!("Cannot perform operation {} - {}", t1, t2)
                            }) 
                        }
                    }
                },
                BinaryOperator::Mul | BinaryOperator::Div => {
                    if left_type == Type::Int && right_type == Type::Int { 
                        ntm.insert(*id, Type::Int);
                        Ok((Type::Int, false)) 
                    }
                    else {
                        Err(Error { loc: *loc, msg: format!("Cannot perform * or / on {}, {}", left_type, right_type) })
                    }
                },
                BinaryOperator::Eq | BinaryOperator::Neq => {
                    if left_type == right_type {
                        ntm.insert(*id, Type::Bool);
                        Ok((Type::Bool, false))
                    }
                    else {
                        Err(Error { 
                            loc: *loc, 
                            msg: format!("Cannot perform == or != on {}, {}", left_type, right_type) 
                        })
                    }
                },
                BinaryOperator::Lt | BinaryOperator::Leq 
                | BinaryOperator::Gt | BinaryOperator::Geq => {
                    if left_type == right_type {
                        ntm.insert(*id, Type::Bool);
                        Ok((Type::Bool, false))
                    }
                    else {
                        Err(Error { 
                            loc: *loc, 
                            msg: format!("Cannot perform <, <=, >, > on {}, {}", left_type, right_type) 
                        })
                    }
                },
                BinaryOperator::And | BinaryOperator::Or => {
                    if left_type == Type::Bool && right_type == Type::Bool { 
                        ntm.insert(*id, Type::Bool);
                        Ok((Type::Bool, false)) 
                    }
                    else {
                        Err(Error { 
                            loc: *loc, 
                            msg: format!("Cannot perform &&, || on {}, {}", left_type, right_type) 
                        })
                    }
                }
            }
        },
        Expr::UnOp{id, loc, operator:op, operand:expr} => {
            let expr_type = 
                match type_check_expr(expr, sym_tab, ntm) {
                    Ok((t, _)) => t,
                    Err(e) => return Err(e),
                };
            match op {
                UnaryOperator::Pos |
                UnaryOperator::Neg => {
                    if expr_type == Type::Int {
                        ntm.insert(*id, Type::Int);
                        Ok((Type::Int, false)) 
                    }
                    else { Err(Error {
                            loc: *loc,
                            msg: format!("Cannot perform - or +{}", expr_type),
                        })
                    } 
                },
                UnaryOperator::Not => {
                    if expr_type == Type::Bool { 
                        ntm.insert(*id, Type::Bool);
                        Ok((Type::Bool, false)) 
                    }
                    else { Err(Error {
                            loc: *loc,
                            msg: format!("Cannot perform !{}", expr_type),
                        })
                    }
                }
            }
        },
        Expr::Call {id, loc, name, args, ret_type} => {
            let mut call_arg_types: Vec<Type> = Vec::new();
            for arg in args.iter_mut() {
                match type_check_expr(arg, sym_tab, ntm) {
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
                                msg: format!("Type mismatch of {} on argument {}. Expected {} but got {}.", name, i, formal_arg, call_arg_types[i])
                            })
                        }
                    }
                    let resolved_type = f_ret_type.as_ref().to_owned();
                    *ret_type = Some(resolved_type.clone());
                    ntm.insert(*id, resolved_type.clone());
                    Ok((resolved_type, false))
                }
                t => Err(Error { 
                    loc: *loc, 
                    msg: format!("Tried to call a {}", t) 
                })
            }
        }
    }
}

fn type_check_stmt (stmt:&mut Box<Stmt>, return_type:Option<&Type>, sym_tab:&mut SymbolTable<Type>, ntm: &mut NodeTypeMap) -> Result<(), Error> {
    match stmt.as_mut() {
        Stmt::Declare{id:_, loc, typ:t, name} => {
            match sym_tab.lookup_current_scope(name) {
                Some(_) => return Err(Error { loc: *loc, msg: format!("Redeclaration of variable {}", name) }),
                None => ()
            }
            sym_tab.define(name, t.to_owned());
            Ok(()) 
        },
        Stmt::Define{id:_, loc, typ:declared_type, name, rhs:expr} => {
            match sym_tab.lookup_current_scope(name) {
                Some(_) => return Err(Error { loc: *loc, msg: format!("Redeclaration of variable {}", name) }),
                None => ()
            }
            let expr_type =
                match type_check_expr(expr, sym_tab, ntm) {
                    Ok((t, _)) => t,
                    Err(err) => return Err(err),
                };
            if &expr_type == declared_type {
                sym_tab.define(name, declared_type.to_owned());
                Ok(())
            }
            else {Err(Error { 
                loc: *loc,
                msg: format!("Variable {} expected type {} but got {}.", name, declared_type, expr_type)
             })}
        },
        Stmt::Assign{id:_, loc, lhs, rhs} => {
            let (lhs_type, lhs_is_lvalue) = 
                match type_check_expr(lhs, sym_tab, ntm) {
                    Ok(t) => t,
                    Err(err) => return Err(err)
                };
            if !lhs_is_lvalue {
                return Err(Error { 
                    loc: *loc, 
                    msg: format!("Left hand side of assignment is not valid l-value.") })
            }

            match lhs_type {
                Type::Arr(_,_) => {
                    return Err(Error { 
                        loc: *loc, 
                        msg: format!("Cannot assign to an entire array of type {}", lhs_type) 
                    });
                },
                Type::Func { name:_, args:_, ret_type:_ } => {
                    return Err(Error { 
                        loc: *loc, 
                        msg: format!("Cannot assign to a function of type {}", lhs_type) 
                    });
                },
                _ => ()
            }

            let (rhs_type, _) =
                match type_check_expr(rhs, sym_tab, ntm) {
                    Ok(t) => t,
                    Err(err) => return Err(err)
                };
            
            if lhs_type == rhs_type { Ok(()) }
            else {
                Err(Error {
                    loc: *loc,
                    msg: format!("Type mismatch in assignment: expected {}, got {}.", lhs_type, rhs_type)
                })
            }
        },
        Stmt::Break{id:_, loc:_} | Stmt::Continue{id:_, loc:_} => Ok(()),
        Stmt::Expr{id:_, loc:_, expr} => {
            match type_check_expr(expr, sym_tab, ntm) {
                Ok(_) => Ok(()),
                Err(err) => Err(err)
            }
        },
        Stmt::PrintByte { id:_, loc, expr } => {
            let (expr_type, _) = type_check_expr(expr, sym_tab, ntm)?;
            if expr_type == Type::Byte {Ok(())}
            else { 
                Err(Error { 
                    loc: *loc, 
                    msg: format!("print_byte expected {} but got {}.", Type::Byte, expr_type) }) 
            }
        },
        Stmt::PrintInt { id:_, loc, expr } => {
            let (expr_type, _) = type_check_expr(expr, sym_tab, ntm)?;
            if expr_type == Type::Int {Ok(())}
            else {
                Err(Error { 
                    loc: *loc, 
                    msg: format!("print_int expected {} but got {}.", Type::Int, expr_type)
                })
            }
        },
        Stmt::PrintString { id:_, loc, expr } => {
            let (expr_type, _) = type_check_expr(expr, sym_tab, ntm)?;
            match &expr_type {
                Type::Ptr(t)
                | Type::Arr(t, _) => {
                    if t.as_ref() == &Type::Byte {Ok(())}
                    else {
                        Err(Error { 
                        loc: *loc, 
                        msg: format!("print_string expected {} but got {}.", Type::Ptr(Box::new(Type::Byte)), expr_type) })
                    }
                },
                _ => {
                    Err(Error { 
                    loc: *loc, 
                    msg: format!("print_string expected {} but got {}.", Type::Ptr(Box::new(Type::Byte)), expr_type) })
                }
            }
        }
        Stmt::Return{id:_, loc, expr} => {
            let ret_type = 
                if let Some(t) = return_type { t } 
                else {return Err(Error { 
                    loc: *loc, 
                    msg: format!("Tried to return in global scope.") });
                };
            match type_check_expr(expr, sym_tab, ntm) {
                Err(err) => Err(err),
                Ok((t, _)) => {
                    if &t == ret_type {Ok(())}
                    else {Err(Error { 
                        loc: *loc, 
                        msg: format!("Expected to return type {} but returned {}.", ret_type, t) })
                    }
                }
            }
        },
        Stmt::If { id:_, loc, cond, true_arm, false_arm } => {
            match type_check_expr(cond, sym_tab, ntm) {
                Ok((t, _)) => {
                    if t != Type::Bool { return Err(Error { loc: *loc, msg: format!("Expected type bool for condition, but got {}", t) }) }
                }
                Err(err) => return Err(err)
            }

            sym_tab.push();
            match type_check_stmt(true_arm, return_type, sym_tab, ntm) {
                Ok(()) => (),
                Err(err) => { sym_tab.pop();return Err(err) }
            }
            sym_tab.pop();

            sym_tab.push();
            match type_check_stmt(false_arm, return_type, sym_tab, ntm) {
                Ok(()) => (),
                Err(err) => { sym_tab.pop(); return Err(err) }
            }
            sym_tab.pop();
            Ok(())
        },
        Stmt::While { id:_, loc, cond, body } => {
            match type_check_expr(cond, sym_tab, ntm) {
                Ok((t, _)) => {
                    if t != Type::Bool { return Err(Error { loc: *loc, msg: format!("Expected type bool for condition, but got {}", t) }) }
                }
                Err(err) => return Err(err)
            }
            sym_tab.push();
            match type_check_stmt(body, return_type, sym_tab, ntm) {
                Ok(()) => (),
                Err(err) => {sym_tab.pop(); return Err(err)}
            }
            sym_tab.pop();
            Ok(())
        },
        Stmt::Block{id:_, loc:_, stmts} => {
            sym_tab.push();
            match collect_fndecl(stmts, sym_tab) {
                Ok(()) => (),
                Err(err) => {sym_tab.pop(); return Err(err)}
            }
            for stmt in stmts.iter_mut() {
                match type_check_stmt(stmt, return_type, sym_tab, ntm) {
                    Err(err) => {sym_tab.pop(); return Err(err) },
                    Ok(()) => ()
                }
            }
            sym_tab.pop();
            Ok(())
        }
        Stmt::FnDecl {id:_, loc:_, name:_, args, ret_type, body } => {
            sym_tab.push();
            for (arg_name, arg_type) in args.iter() {
                sym_tab.define(arg_name.as_str(), arg_type.to_owned());
            }
            match type_check_stmt(body, Some(ret_type), sym_tab, ntm) {
                Ok(()) => (),
                Err(err) => {sym_tab.pop(); return Err(err)},
            }
            sym_tab.pop();
            Ok(())
        }
    }
}

fn collect_fndecl (stmts:&Vec<Box<Stmt>>, sym_tab:&mut SymbolTable<Type>) -> Result<(), Error> {
    for stmt in stmts.iter() {
        match stmt.as_ref() {
            Stmt::FnDecl {id:_, loc, name, args, ret_type, body:_ } => {
                match sym_tab.lookup_current_scope(name) {
                    Some(_) => return Err(Error { loc: *loc, msg: format!("Redeclaration of function {}", name) }),
                    None => ()
                }
                let arg_types = 
                    args
                    .iter()
                    .map(|x| Box::new(x.1.clone()))
                    .collect();
                let f_type = Type::Func { name:name.to_owned(), args: arg_types, ret_type: Box::new(ret_type.to_owned()) };
                sym_tab.define(name, f_type);
            }
            _ => ()
        }
    }
    Ok(())
}

pub fn type_check (stmts:&mut Vec<Box<Stmt>>, ntm: &mut NodeTypeMap) -> Result<(), Error> {
    let mut sym_tab = SymbolTable::new();
    sym_tab.push();
    match collect_fndecl(stmts, &mut sym_tab) {
        Ok(()) => (),
        Err(err) => return Err(err)   
    }
    for stmt in stmts.iter_mut() {
        match type_check_stmt(stmt, None, &mut sym_tab, ntm) {
            Err(err) => return Err(err),
            Ok(()) => ()
        }
    }
    sym_tab.pop();
    Ok(())
}