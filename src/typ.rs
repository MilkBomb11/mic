#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int, 
    Bool,
    Byte,
    Ptr(Box<Type>),
    Arr(Box<Type>, usize),
    Func {name:String, args:Vec<Box<Type>>, ret_type:Box<Type>},
}