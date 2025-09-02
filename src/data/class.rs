use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::Value;

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
    class: Rc<Class>,
    fields: RefCell<HashMap<Rc<String>, Value>>,
}

impl Instance {
    pub fn new(class: Rc<Class>) -> Self {
        Self {
            class,
            fields: Default::default(),
        }
    }

    pub fn get_field(&self, name: Rc<String>) -> Option<Value> {
        self.fields.borrow().get(&name).cloned()
    }

    pub fn set_field(&self, name: Rc<String>, v: Value) {
        self.fields.borrow_mut().insert(name, v);
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{} instance>", self.class.name)
    }
}
