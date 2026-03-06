use std::{fmt::Display, vec};


type Reg = String;

#[derive(Debug, Clone)]
pub enum Operand {
    Reg(Reg),
    Imm(i32),
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Imm(n) => write!(f, "{}", n),
            Operand::Reg(r) => write!(f, "{}", r), 
        }
    }
}

#[derive(Debug, Clone)]
pub enum Lbl {
    Name(String),
    Resolved(usize),
}

impl Display for Lbl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lbl::Name(label) => write!(f, "<{}>", label),
            Lbl::Resolved(label) => write!(f, "{}", label),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instr {
    Alloc{dest:Reg, size:usize},
    Set{dest:Reg, src:Operand},
    BinOp{dest:Reg, left:Operand, operator:BinaryOp, right:Operand},
    UnOp{dest:Reg, operator:UnaryOp, operand:Operand},
    Load{size:Size, dest:Reg, src:Reg},
    Store{size:Size, src:Operand, dest:Reg,},
    Label{label:Lbl},
    Goto{dest:Lbl},
    GotoIf{cond:Operand, dest:Lbl},
    GotoIfFalse{cond:Operand, dest:Lbl},
    Call{dest:Reg, name:String, args:Vec<Operand>},
    FnDecl{name:String, params:Vec<String>, body:Vec<Instr>},
    Ret{operand:Operand},
    PrintInt{operand:Operand},
    PrintByte{operand:Operand},
    PrintString{src:Operand},
}

impl Instr {
    pub fn get_def(&self) -> Option<&str> {
        match self {
            Instr::Alloc { dest, .. }
            | Instr::BinOp { dest, .. }
            | Instr::UnOp { dest, .. }
            | Instr::Call { dest, .. }
            | Instr::Load { dest, .. }
            | Instr::Set { dest, .. } => Some(dest.as_str()),
            _ => None
        }
    }

    pub fn get_use(&self) -> Vec<&str> {
        match self {
            Instr::Set { src, .. } => {
                let mut uses = vec![];
                if let Operand::Reg(r) = src { uses.push(r.as_str()); }
                uses
            }
            Instr::BinOp { left, right, ..} => {
                let mut uses = vec![];
                if let Operand::Reg(r1) = left { uses.push(r1.as_str()); }
                if let Operand::Reg(r2) = right { uses.push(r2.as_str()); }
                uses
            },
            Instr::Ret { operand }
            | Instr::PrintInt { operand }
            | Instr::PrintByte { operand }
            | Instr::UnOp { operand, .. } => {
                let mut uses = vec![];
                if let Operand::Reg(r) = operand { uses.push(r.as_str()); }
                uses
            },
            Instr::Load { src, .. } => vec![src.as_str()],
            Instr::Store { src, dest, .. } => {
                let mut uses = vec![dest.as_str()];
                if let Operand::Reg(r) = src { uses.push(r.as_str()); }
                uses
            },
            Instr::GotoIf { cond, .. }
            | Instr::GotoIfFalse { cond, .. } => {
                let mut uses = vec![];
                if let Operand::Reg(r) = cond { uses.push(r.as_str()); }
                uses
            },
            Instr::Call { args, .. } => {
                let mut uses = vec![];
                for operand in args.iter() {
                    if let Operand::Reg(r) = operand {
                        uses.push(r.as_str());
                    }
                }
                uses
            },
            Instr::PrintString { src } => {
                let mut uses = vec![];
                if let Operand::Reg(r) = src { uses.push(r.as_str()); }
                uses
            }
            _ => vec![]
        }
    }

    fn fmt_with_depth(&self, f: &mut std::fmt::Formatter<'_>, depth: usize) -> std::fmt::Result {
        let padding = " ".repeat(depth*4);
        match self {
            Instr::Alloc { dest, size } => write!(f, "{}{} = alloc({})", padding, dest, size),
            Instr::BinOp { 
                dest, 
                left, 
                operator, 
                right } => write!(f, "{}{} := {} {} {}", padding, dest, left, operator, right),
            Instr::UnOp { dest, operator, operand } => write!(f, "{}{} := {} {}", padding, dest, operator, operand),
            Instr::Goto { dest } => write!(f, "{}goto {}", padding, dest),
            Instr::GotoIf { cond, dest } => write!(f, "{}if {} goto {}", padding, cond, dest),
            Instr::GotoIfFalse { cond, dest } => write!(f, "{}if not {} goto {}", padding, cond, dest),
            Instr::Set { dest, src } => write!(f, "{}{} := {}",padding, dest, src),
            Instr::Ret { operand } => write!(f, "{}ret {}",padding, operand),
            Instr::PrintInt { operand } => write!(f, "{}printint {}", padding, operand),
            Instr::PrintByte { operand } => write!(f, "{}printchr {}", padding, operand),
            Instr::PrintString { src } => write!(f, "{}printstr {}", padding, src),
            Instr::Load { size, dest, src } => write!(f, "{}{} := load {} [{}]", padding, dest, src, size),
            Instr::Store { size, dest, src } => write!(f, "{}store {} -> ({}) [{}]", padding, src, dest, size),
            Instr::Label { label } => write!(f, "{}label {}", padding, label),
            Instr::Call { dest, name, args } => {
                let args: Vec<String> = 
                    args
                    .iter()
                    .map(|o| o.to_string())
                    .collect();
                write!(f, "{}{} := call ({}|{})",padding, dest, name, args.join(","))
            }
            Instr::FnDecl { name, params, body } => {
                writeln!(f, "{}fn {}({}):", padding, name, params.join(","), )?;
                for instr in body.iter() {
                    instr.fmt_with_depth(f, depth + 1)?;
                    writeln!(f)?;
                }
                write!(f, "{}", padding)
            }
        }
    }
}

impl Display for Instr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_with_depth(f, 0)
    }
}



#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add, Sub, Mul, Div,
    Lt, Leq, Gt, Geq,
    Eq, Neq,
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Lt => write!(f, "<"),
            BinaryOp::Leq => write!(f, "<="),
            BinaryOp::Gt => write!(f, ">"),
            BinaryOp::Geq => write!(f, ">="),
            BinaryOp::Neq => write!(f, "!="),
            BinaryOp::Eq => write!(f, "=="),
        }
    }
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg, Pos, Not,
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOp::Neg => write!(f, "-"),
            UnaryOp::Pos => write!(f, "+"),
            UnaryOp::Not => write!(f, "!"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Size {
    Double,
    Word,
    Byte,
}

impl Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Size::Double => write!(f, "8 bytes"),
            Size::Word => write!(f, "4 bytes"),
            Size::Byte => write!(f, "byte"),
        }
    }
}

pub struct IRBuilder {
    pub instrs: Vec<Instr>,
    reg_counter: usize,
    label_counter: usize,
}

impl IRBuilder {
    pub fn new() -> Self {
        Self { 
            instrs: Vec::new(), 
            reg_counter: 0, 
            label_counter: 0 
        }
    }

    pub fn get_counters (&self) -> (usize, usize) {
        (self.reg_counter, self.label_counter)
    }

    pub fn set_counters (&mut self, counters:(usize, usize)) -> () {
        let (rc, lc) = counters;
        self.reg_counter = rc;
        self.label_counter = lc;
    }

    pub fn emit(&mut self, instr: Instr) {
        self.instrs.push(instr);
    }

    pub fn new_reg(&mut self) -> String {
        let reg = format!("%r{}", self.reg_counter);
        self.reg_counter += 1;
        reg
    }

    pub fn new_label(&mut self) -> String {
        let label = format!(".L{}", self.label_counter);
        self.label_counter += 1;
        label
    } 
}