use mic::{ir::IRBuilder, node_id_assigner::{IdBuilder, assign_id}, parser, report_error, report_parse_error};
use mic::{translate::translate_stmts, typ::Type, type_check::type_check};
use std::{collections::HashMap, env, fs::File, io::Read, path::Path};
fn main() {
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

    run(&source);
}

fn run(source: &str) -> () {
    let program_parser = parser::ProgramParser::new();

    match program_parser.parse(source) {
        Ok(ast) => {
            let mut id_builder = IdBuilder::new();
            let mut ast = ast;
            assign_id(&mut ast, &mut id_builder);

            println!("Successfully parsed {} statements!", ast.len());
            //println!("{:#?}", ast);
            let mut node_type_map: HashMap<usize, Type> = HashMap::new();
            match type_check(&mut ast, &mut node_type_map) {
                Ok(()) => {
                    println!("Type-check successful for {} statements!", ast.len());
                    // println!("{:#?}", ast);
                    // println!("{:?}", node_type_map);
                },
                Err(err) => { report_error(source, err); return;}
            }
            let mut ir_builder = IRBuilder::new();
            match translate_stmts(&ast, &mut ir_builder, &node_type_map) {
                Ok(()) => {
                    println!("Translation successful!");
                    for instr in ir_builder.instrs.iter() {
                        println!("{}", instr);
                    }
                },
                Err(err) => { report_error(source, err); return;}
            }
        },
        Err(err) => { report_parse_error(&source, err); return;},
    }
}