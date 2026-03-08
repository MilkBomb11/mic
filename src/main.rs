use mic::capture::capture;
use mic::flatten::flatten;
use mic::goto_cleanup::GotoCleanup;
use mic::qbe::QbeGenerator;
use mic::return_check::return_check;
use mic::{function_renamer::FunctionRenamer, ir::IRBuilder, report_error, report_parse_error};
use mic::{node_id_assigner::{IdBuilder, assign_id}, parser, program_printer::ProgramPrinter};
use mic::{translate::translate_stmts, typ::Type, type_check::type_check};
use std::fs;
use std::process::{Command, exit};
use std::{collections::HashMap, env, fs::File, io::Read};
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 && args.len() != 4 {
        eprintln!("Usage: {} [-l] <src-file> <dest-file>", args[0]);
        exit(1);
    }

    let is_log = if args[1] == "-l" { true } else { false };
    let src_file_path = if args.len() == 3 { &args[1] } else { &args[2] };
    let dest_file_path = if args.len() == 3 { &args[2] } else { &args[3] };

    let mut file = match File::open(src_file_path) {
        Ok(file) => file,
        Err(e) => panic!("Failed to open file because: {}", e),  
    };

    let mut source:String = String::new();
    match file.read_to_string(&mut source) {
        Err(e) => panic!("Failed to read file because: {}", e),
        Ok(_) => (),
    }

    match run(&source, is_log) {
        Ok(out) => {
            let temp_ssa = format!("{}.ssa", dest_file_path);
            let temp_asm = format!("{}.s", dest_file_path);
            let temp_c = format!("{}_runtime.c", dest_file_path);
            fs::write(temp_ssa.as_str(), out).expect("Failed to write temporary .ssa file");

            let runtime_c_code = r#"
#include <stdio.h>
void print_int(long val) { printf("%ld", val); }
void print_byte(long val) { printf("%c", (char)val); }
void print_string(char* val) { printf("%s", val); }
"#;
            fs::write(&temp_c, runtime_c_code).expect("Failed to write runtime.c");

            let qbe_status = Command::new("qbe")
                .arg("-o")
                .arg(&temp_asm)
                .arg(&temp_ssa)
                .status()
                .expect("Failed to execute QBE. Is it installed and in your PATH?");

            if !qbe_status.success() {
                eprintln!("QBE compilation failed!");
                exit(1);
            }

            let cc_status = Command::new("cc") 
                .arg(&temp_asm)
                .arg(&temp_c)
                .arg("-o")
                .arg(dest_file_path)
                .status()
                .expect("Failed to execute C compiler.");

            if !cc_status.success() {
                eprintln!("Linking failed!");
                exit(1);
            }

            if !is_log {
                let _ = fs::remove_file(&temp_ssa);
                let _ = fs::remove_file(&temp_asm);
                let _ = fs::remove_file(&temp_c);
            }

            println!("Successfully compiled to executable: {}", dest_file_path);
        }
        Err(()) => (),
    }

}

fn run(source: &str, is_log: bool) -> Result<String,()> {
    let program_parser = parser::ProgramParser::new();

    match program_parser.parse(source) {
        Ok(ast) => {
            let mut id_builder = IdBuilder::new();
            let mut ast = ast;
            assign_id(&mut ast, &mut id_builder);

            if is_log { println!("Successfully parsed {} statements!", ast.len()); }
            //println!("{:#?}", ast);
            let mut node_type_map: HashMap<usize, Type> = HashMap::new();
            match type_check(&mut ast, &mut node_type_map) {
                Ok(()) => {
                    if is_log { println!("Type-check successful!"); }
                    // println!("{:#?}", ast);
                    // println!("{:?}", node_type_map);
                },
                Err(err) => { report_error(source, err); return Err(());}
            }

            match return_check(&ast) {
                Ok(_) =>  if is_log { println!("Return-check successful!") },
                Err(err) => { report_error(source, err); return Err(()); }
            }

            let mut ir_builder = IRBuilder::new();
            match translate_stmts(&ast, &mut ir_builder, &node_type_map) {
                Ok(()) => {
                    if is_log { println!("Translation successful!"); }
                    //println!("{}", ProgramPrinter(&ir_builder.instrs))
                },
                Err(err) => { report_error(source, err); return Err(());}
            }
            let mut ir_builder = ir_builder.epilogue();

            let mut function_renamer: FunctionRenamer = FunctionRenamer::new();
            function_renamer.traverse(&mut ir_builder.instrs);
            if is_log { println!("Function renaming successful!"); }
            //println!("{}", ProgramPrinter(&ir_builder.instrs));

            let mut flattened_ir_builder = IRBuilder::new();
            flatten(&ir_builder.instrs, &mut flattened_ir_builder);
            if is_log { println!("Flattening successful!"); }
            //println!("{}", ProgramPrinter(&flattened_ir_builder.instrs));

            capture(&mut flattened_ir_builder.instrs);
            if is_log { println!("Capturing successful!"); }
            //println!("{}", ProgramPrinter(&flattened_ir_builder.instrs));

            let mut ir_builder = IRBuilder::new();
            let mut goto_cleanup = GotoCleanup::new(); 
            goto_cleanup.cleanup(&flattened_ir_builder.instrs, &mut ir_builder,);
            if is_log { println!("Goto cleanup successful!"); }
            if is_log { println!("{}", ProgramPrinter(&ir_builder.instrs)); }

            let mut qbe_gen = QbeGenerator::new();
            let out = qbe_gen.generate(&ir_builder.instrs);
            Ok(out)
        },
        Err(err) => { report_parse_error(&source, err); return Err(());},
    }
}