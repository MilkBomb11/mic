struct Ctx {
    break_label: String,
    continue_label: String,
}

pub struct  JumpContext {
    contexts: Vec<Ctx>,
}

impl JumpContext {
    pub fn new() -> Self {
        Self {
            contexts: Vec::new()
        }
    }

    pub fn push(&mut self, break_label:&str, continue_label:&str) -> () {
        self.contexts.push(Ctx { 
            break_label: break_label.to_owned(), 
            continue_label: continue_label.to_owned()
        });
    }

    pub fn pop(&mut self) -> () {
        self.contexts.pop();
    }

    pub fn lookup_break(&self) -> Option<&str> {
        if let Some(ctx) = self.contexts.last() {
            return Some(ctx.break_label.as_str())
        }
        None
    }

    pub fn lookup_continue(&self) -> Option<&str> {
        if let Some(ctx) = self.contexts.last() {
            return Some(ctx.continue_label.as_str())
        }
        None
    }
}