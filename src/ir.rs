
type Reg = String;

#[derive(Debug)]
pub enum Operand {
    Reg(Reg),
    Imm(i32),
}

#[derive(Debug)]
pub enum Lbl {
    Name(String),
    Resolved(usize),
}

#[derive(Debug)]
pub enum Instr {
    Alloc(Reg, usize),
    MemCpy(Reg, Reg, usize),
    Set(Reg, Operand),
    BinOp(Reg, Operand, BinaryOp, Operand),
    UnOp(Reg, UnaryOp, Operand),
    Load(Size, Reg, Reg),
    Store(Size, Reg, Operand),
    Label(Lbl),
    Goto(Lbl),
    GotoIf(Operand, Lbl),
    GotoIfFalse(Operand, Lbl),
    Call(String, Vec<Operand>),
    FnDecl {name:String, params:String, body:Vec<Instr>}
}

#[derive(Debug)]
pub enum BinaryOp {
    Add, Sub, Mul, Div,
    Lt, Leq, Gt, Geq,
    Eq, Neq,
}

#[derive(Debug)]
pub enum UnaryOp {
    Neg, Pos, Not,
}

#[derive(Debug)]
pub enum Size {
    Double,
    Word,
    Byte,
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