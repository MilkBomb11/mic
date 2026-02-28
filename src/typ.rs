use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int, 
    Bool,
    Byte,
    Ptr(Box<Type>),
    Arr(Box<Type>, usize),
    Func {name:String, args:Vec<Box<Type>>, ret_type:Box<Type>},
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Bool => write!(f, "bool"),
            Type::Byte => write!(f, "byte"),
            Type::Arr(t, size) => write!(f, "{}[{}]", t, size),
            Type::Ptr(t) => write!(f, "ptr<{}>", t),
            Type::Func { name:_, args, ret_type } => {
                let arg_strings: Vec<String> = 
                    args
                    .iter()
                    .map(|t| t.to_string())
                    .collect();
                write!(f, "(({}) -> {})", arg_strings.join(","), ret_type.to_string())
            }
        }
    }

}