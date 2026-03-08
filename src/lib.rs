pub mod ast;
pub mod typ;
pub mod value;
pub mod node_id_assigner;
pub mod symbol_table;
pub mod jump_context;
pub mod type_check;
pub mod return_check;
pub mod ir;
pub mod translate;
pub mod function_renamer;
pub mod flatten;
pub mod program_printer;
pub mod capture;
pub mod goto_cleanup;
pub mod qbe;

use std::fmt::{Debug};

use lalrpop_util::lalrpop_mod;
use lalrpop_util::ParseError;

lalrpop_mod!(pub parser);

#[derive(Debug, Clone)]
pub struct Error {
    loc: usize,
    msg: String
}

pub fn get_line_col (source:&str, offset:usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, c) in source.char_indices() {
        if i == offset {break;}
        if c == '\n' {
            line += 1;
            col = 1;
        }
        else { col += 1; }
    }
    (line, col)
}

pub fn report_parse_error<'a, T> (source:&'a str, err:ParseError<usize, T, Error>) -> () 
where 
    T: Debug,
{
    match err {
        ParseError::InvalidToken { location } => {
            let (line, col) = get_line_col(source, location);
            println!("Syntax error at line {}, column {}.", line, col);
            println!("Found invalid token.");
        }
        ParseError::ExtraToken { token: (start, token, _) } => {
            let (line, col) = get_line_col(source, start);
            println!("Syntax error at line {}, column {}.", line, col);
            println!("Found extra token {:?}", token);
        }
        ParseError::UnrecognizedToken { token: (start, token, _), expected } => {
            let (line, col) = get_line_col(source, start);
            println!("Syntax error at line {}, column {}.", line, col);
            println!("Unexpected token {:?}. Expected the following.", token);
            println!("{}", expected.join(","));
        },
        ParseError::UnrecognizedEof { location, expected } => {
            let (line, col) = get_line_col(source, location);
            println!("Syntax error at line {}, column {}.", line, col);
            println!("Unexpected EOF. Expected the following.");
            println!("{}", expected.join(","));
        }
        ParseError::User { error: Error {loc, msg}} => {
            let (line, col) = get_line_col(source, loc);
            println!("Syntax error at line {}, column {}.", line, col);
            println!("{}", msg);
        }
    }
}

pub fn report_error (source:&str, err:Error) -> () {
    let (line, col) = get_line_col(source, err.loc);
    println!("Error at line {}, column {}", line, col);
    println!("{}", err.msg);
}