pub mod ast;
pub mod typ;
pub mod value;

use std::fmt::{Display, Debug};

use lalrpop_util::lalrpop_mod;
use lalrpop_util::ParseError;

lalrpop_mod!(pub parser);

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

pub fn report_error<'a, T, E> (source:&'a str, err:ParseError<usize, T, E>) -> () 
where 
    T: Debug,
    E: Display,
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
        ParseError::User { error } => {
            println!("{}", error);
        }
    }
}