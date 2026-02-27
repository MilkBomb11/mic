use std::{collections::HashMap, fmt::Debug};
pub struct SymbolTable<V> 
where V: Debug
{
    table:Vec<HashMap<String, V>>
}

impl<V> SymbolTable<V> 
where V: Debug 
{
    pub fn new() -> Self {
        Self { table: Vec::new() }
    }

    pub fn push(&mut self) -> () {
        self.table.push(HashMap::new());
    }

    pub fn pop(&mut self) -> () {
        self.table.pop();
    }

    pub fn define(&mut self, k:&str, v:V) -> () {
        match self.table.last_mut() {
            None => panic!("Tried to define {} {:?} to an empty symbol table", k, v),
            Some(map) => {
                map.insert(k.to_string(), v);
            }
        }
    }

    pub fn lookup(&self, k:&str) -> Option<&V> {
        for map in self.table.iter().rev() {
            if let Some(v) = map.get(k) {return Some(v);}
        }
        None
    }

    pub fn lookup_current_scope(&self, k:&str) -> Option<&V> {
        match self.table.last() {
            None => None,
            Some(map) => { map.get(k) }
        }
    }
}