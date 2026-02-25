use mic::{parser, report_error};
use std::{env, fs::File, io::Read, path::Path};
fn main() {
    // Initialize the new Program level parser
    let program_parser = parser::ProgramParser::new();

    let args: Vec<String> = env::args().collect();
    let path = 
        match args.len() {
            1 => Path::new("./tests/test.txt"),
            2 => Path::new(&args[1]),
            _ => panic!("Usage: mic [src-file]"),
        };

    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) => panic!("Failed to open file because: {}", e),  
    };

    let mut source:String = String::new();
    match file.read_to_string(&mut source) {
        Err(e) => panic!("Failed to read file because: {}", e),
        Ok(_) => (),
    }

    match program_parser.parse(&source) {
        Ok(ast) => {
            println!("Successfully parsed {} statements!", ast.len());
            println!("{:#?}", ast);
        },
        Err(e) => { report_error(&source, e); },
    }
}