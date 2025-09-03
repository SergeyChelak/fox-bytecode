use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::Value;

#[derive(Debug)]
pub struct Class {
    name: Rc<String>,
    methods: RefCell<HashMap<Rc<String>, Value>>,
}

impl Class {
    pub fn new(name: Rc<String>) -> Self {
        Self {
            name,
            methods: Default::default(),
        }
    }

    pub fn add_method(&self, name: Rc<String>, value: Value) {
        // TODO: replace with try_borrow_mut
        self.methods.borrow_mut().insert(name, value);
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
        // TODO: replace with try_borrow
        self.fields.borrow().get(&name).cloned()
    }

    pub fn set_field(&self, name: Rc<String>, v: Value) {
        // TODO: replace with try_borrow_mut
        self.fields.borrow_mut().insert(name, v);
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{} instance>", self.class.name)
    }
}
