use crate::ir::{BinaryOp, Instr, Lbl, Operand, Size, UnaryOp};
use std::fmt::Write;

pub struct QbeGenerator {
    out: String,
    fallthrough_count: usize,
    is_terminated: bool,
}

impl QbeGenerator {
    pub fn new() -> Self {
        Self { 
            out: String::new(), 
            fallthrough_count: 0,
            is_terminated: false, 
        }
    }

    pub fn generate(&mut self, instrs: &[Instr]) -> String {
        for instr in instrs.iter() {
            self.emit_instr(instr);
        }
        self.out.clone()
    }

    fn pure_name(s: &str) -> String {
        s.replace("%", "")
         .replace("$", "")
         .replace("@", "")
         .replace("#", "_")
    }

    fn to_local_reg(s: &str) -> String {
        format!("%{}", Self::pure_name(s))
    }

    fn format_operand(operand: &Operand) -> String {
        match operand {
            Operand::Imm(n) => n.to_string(),
            Operand::Reg(r) => format!("{}", Self::to_local_reg(r.as_str())) 
        }
    }

    fn sanitize(name: &str) -> String {
        name.replace("#", "_").replace("$", "_")
    }

    fn new_fallthrough(&mut self) -> String {
        let ft = format!("@.ft.{}", self.fallthrough_count);
        self.fallthrough_count += 1;
        ft
    }

    fn emit_instr(&mut self, instr: &Instr) -> () {
        if self.is_terminated {
            match instr {
                Instr::Label { .. } | Instr::FnDecl { .. } => {}
                _ => return, 
            }
        }

        match instr {
            Instr::FnDecl { name, params, body } => {
                let safe_name = Self::pure_name(name);
                let qbe_params: Vec<String> = 
                    params
                    .iter()
                    .map(|param| format!("l {}", Self::to_local_reg(param)))
                    .collect();
                let export_kw = if safe_name == "main_0" { "export " } else { "" };
                let safe_name = if safe_name == "main_0" { "main" } else { &safe_name };

                writeln!(self.out, "{}function l ${}({}) {{", export_kw, safe_name, qbe_params.join(", ")).unwrap();
                writeln!(self.out, "@start").unwrap();

                self.is_terminated = false;
                for b_instr in body {
                    self.emit_instr(b_instr);
                }

                if !self.is_terminated {
                    writeln!(self.out, "    ret 0").unwrap();
                    self.is_terminated = true;
                }
                writeln!(self.out, "}}\n").unwrap();
            }
            Instr::Alloc { dest, size } => {
                writeln!(self.out, "    {} =l alloc8 {}", Self::sanitize(dest), size).unwrap();
            }
            Instr::Store { size, dest, src } => {
                let store_inst = match size {
                    Size::Byte => "storeb",
                    Size::Word => "storew",
                    Size::Long => "storel",
                };
                writeln!(self.out, "    {} {}, {}", store_inst, Self::format_operand(src), Self::to_local_reg(dest)).unwrap();
            }
            Instr::Load { size, dest, src } => {
                let load_inst = match size {
                    Size::Byte => "loadub",
                    Size::Word => "loadw",  
                    Size::Long => "loadl",
                };
                writeln!(self.out, "    {} =l {} {}", Self::to_local_reg(dest), load_inst, Self::to_local_reg(src)).unwrap();
            }
            Instr::Set { dest, src } => {
                writeln!(self.out, "    {} =l copy {}", Self::sanitize(dest), Self::format_operand(src)).unwrap();
            }
            Instr::Label { label } => {
                let l_name = match label {
                    Lbl::Name(l_name) => l_name.to_owned(),
                    Lbl::Resolved(r) => r.to_string(),
                };
                writeln!(self.out, "@{}", Self::sanitize(l_name.as_str())).unwrap();
                self.is_terminated = false;
            }
            Instr::Goto { dest } => {
                let l_name = match dest {
                    Lbl::Name(l_name) => l_name.to_owned(),
                    Lbl::Resolved(r) => r.to_string(),
                };
                writeln!(self.out, "    jmp @{}", Self::sanitize(l_name.as_str())).unwrap();
                self.is_terminated = true;
            }
            Instr::GotoIf { cond, dest } => {
                let ft = self.new_fallthrough();
                let l_name = match dest {
                    Lbl::Name(l_name) => l_name.to_owned(),
                    Lbl::Resolved(r) => r.to_string(),
                };
                writeln!(self.out, "    jnz {}, @{}, {}", Self::format_operand(cond), Self::sanitize(l_name.as_str()), ft).unwrap();
                writeln!(self.out, "{}", ft).unwrap();
                self.is_terminated = false;
            }
            Instr::GotoIfFalse { cond, dest } => {
                let ft = self.new_fallthrough();
                let l_name = match dest {
                    Lbl::Name(l_name) => l_name.to_owned(),
                    Lbl::Resolved(r) => r.to_string(),
                };
                writeln!(self.out, "    jnz {}, {}, @{}", Self::format_operand(cond), ft, Self::sanitize(l_name.as_str())).unwrap();
                writeln!(self.out, "{}", ft).unwrap();
                self.is_terminated = false;
            }
            Instr::Call { dest, name, args } => {
                let qbe_args: Vec<String> = args.iter().map(|a| format!("l {}", Self::format_operand(a))).collect();
                writeln!(self.out, "    {} =l call ${}({})", Self::sanitize(dest), Self::sanitize(name), qbe_args.join(", ")).unwrap();
            }
            Instr::Ret { operand } => {
                writeln!(self.out, "    ret {}", Self::format_operand(operand)).unwrap();
                self.is_terminated = true;
            }
            Instr::BinOp { dest, left, operator, right } => {
                let dest_san = Self::pure_name(dest);
                let l_op = Self::format_operand(left);
                let r_op = Self::format_operand(right);
                
                let (is_cmp, op_str) = match operator {
                    BinaryOp::Add => (false, "add"),
                    BinaryOp::Sub => (false, "sub"),
                    BinaryOp::Mul => (false, "mul"),
                    BinaryOp::Div => (false, "div"),
                    BinaryOp::Eq  => (true, "ceql"),
                    BinaryOp::Neq => (true, "cnel"),
                    BinaryOp::Lt  => (true, "csltl"),
                    BinaryOp::Leq => (true, "cslel"),
                    BinaryOp::Gt  => (true, "csgtl"),
                    BinaryOp::Geq => (true, "csgel"),
                };

                if is_cmp {
                    writeln!(self.out, "    %_cmp_{} =w {} {}, {}", dest_san, op_str, l_op, r_op).unwrap();
                    writeln!(self.out, "    %{} =l extuw %_cmp_{}", dest_san, dest_san).unwrap();
                } else {
                    writeln!(self.out, "    %{} =l {} {}, {}", dest_san, op_str, l_op, r_op).unwrap();
                }
            }
            Instr::UnOp { dest, operator, operand } => {
                let dest_san = Self::pure_name(dest);
                let op_fmt = Self::format_operand(operand);
                match operator {
                    UnaryOp::Pos => writeln!(self.out, "    %{} =l copy {}", dest_san, op_fmt).unwrap(),
                    UnaryOp::Neg => writeln!(self.out, "    %{} =l sub 0, {}", dest_san, op_fmt).unwrap(),
                    UnaryOp::Not => {
                        writeln!(self.out, "    %_cmp_{} =w ceql {}, 0", dest_san, op_fmt).unwrap();
                        writeln!(self.out, "    %{} =l extuw %_cmp_{}", dest_san, dest_san).unwrap();
                    }
                }
            }

            Instr::PrintByte { operand } => {
                writeln!(self.out, "    call $print_byte(l {})", Self::format_operand(operand)).unwrap();
            }
            Instr::PrintInt { operand } => {
                writeln!(self.out, "    call $print_int(l {})", Self::format_operand(operand)).unwrap();
            }
            Instr::PrintString { src } => {
                writeln!(self.out, "    call $print_string(l {})", Self::format_operand(src)).unwrap();
            }
            Instr::GetByte { dest_addr } => {
                writeln!(self.out, "    call $get_byte(l {})", Self::format_operand(dest_addr)).unwrap();
            }
            Instr::GetInt { dest_addr } => {
                writeln!(self.out, "    call $get_int(l {})", Self::format_operand(dest_addr)).unwrap();
            }
        }
    }
}