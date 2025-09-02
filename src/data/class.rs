use std::{fmt::Display, rc::Rc};

#[derive(Debug)]
pub struct Class {
    name: Rc<String>,
}

impl Class {
    pub fn new(name: Rc<String>) -> Self {
        Self { name }
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<class {}>", self.name)
    }
}

#[derive(Debug)]
pub struct Instance {
    //
}
