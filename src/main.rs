use mic::capture::capture;
use mic::flatten::flatten;
use mic::goto_cleanup::GotoCleanup;
use mic::qbe::QbeGenerator;
use mic::return_check::return_check;
use mic::{function_renamer::FunctionRenamer, ir::IRBuilder, report_error, report_parse_error};
use mic::{node_id_assigner::{IdBuilder, assign_id}, parser, program_printer::ProgramPrinter};
use mic::{translate::translate_stmts, typ::Type, type_check::type_check};
use std::io::Write;
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

    match run(&source) {
        Ok(out) => {
            let mut out_file = 
                match File::create("./tests/output.ssa") {
                    Ok(f) => f,
                    Err(e) => panic!("Failed to create file because: {}", e)
                };
            write!(out_file, "{}", out).unwrap()
        }
        Err(()) => (),
    }

}

fn run(source: &str) -> Result<String,()> {
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
                Err(err) => { report_error(source, err); return Err(());}
            }

            match return_check(&ast) {
                Ok(_) => println!("Return-check successful!"),
                Err(err) => { report_error(source, err); return Err(()); }
            }

            let mut ir_builder = IRBuilder::new();
            match translate_stmts(&ast, &mut ir_builder, &node_type_map) {
                Ok(()) => {
                    println!("Translation successful!");
                    //println!("{}", ProgramPrinter(&ir_builder.instrs))
                },
                Err(err) => { report_error(source, err); return Err(());}
            }
            let mut ir_builder = ir_builder.epilogue();

            let mut function_renamer: FunctionRenamer = FunctionRenamer::new();
            function_renamer.traverse(&mut ir_builder.instrs);
            println!("Function renaming successful!");
            //println!("{}", ProgramPrinter(&ir_builder.instrs));

            let mut flattened_ir_builder = IRBuilder::new();
            flatten(&ir_builder.instrs, &mut flattened_ir_builder);
            println!("Flattening successful!");
            //println!("{}", ProgramPrinter(&flattened_ir_builder.instrs));

            capture(&mut flattened_ir_builder.instrs);
            println!("Capturing successful!");
            //println!("{}", ProgramPrinter(&flattened_ir_builder.instrs));

            let mut ir_builder = IRBuilder::new();
            let mut goto_cleanup = GotoCleanup::new(); 
            goto_cleanup.cleanup(&flattened_ir_builder.instrs, &mut ir_builder,);
            println!("Goto cleanup successful!");
            println!("{}", ProgramPrinter(&ir_builder.instrs));

            let mut qbe_gen = QbeGenerator::new();
            let out = qbe_gen.generate(&ir_builder.instrs);
            Ok(out)
        },
        Err(err) => { report_parse_error(&source, err); return Err(());},
    }
}