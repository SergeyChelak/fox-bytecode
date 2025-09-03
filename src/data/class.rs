use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{Closure, Value};

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

    pub fn get_method(&self, name: &Rc<String>) -> Option<Value> {
        // TODO: replace with try_borrow
        self.methods.borrow().get(name).cloned()
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
pub struct BoundMethod {
    receiver: Value,
    method: Rc<Closure>,
}

impl BoundMethod {
    pub fn new(receiver: Value, method: Rc<Closure>) -> Self {
        Self { receiver, method }
    }

    pub fn closure(&self) -> Rc<Closure> {
        self.method.clone()
    }

    pub fn receiver_owned(&self) -> Value {
        self.receiver.clone()
    }
}

impl Display for BoundMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<bound {} for {}>", self.method, self.receiver)
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

    pub fn class(&self) -> Rc<Class> {
        self.class.clone()
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
