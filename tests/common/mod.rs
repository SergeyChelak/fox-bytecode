extern crate fox_bytecode;

use std::{cell::RefCell, rc::Rc};

use fox_bytecode::{Probe, interpret};

pub fn str_to_code_ref(input: &str) -> Rc<Vec<char>> {
    Rc::new(input.chars().collect())
}

pub fn interpret_using_probe(input: &str) -> Rc<RefCell<Probe>> {
    let code_ref = str_to_code_ref(input);
    let machine_io = Probe::new();
    let io_ref = Rc::new(RefCell::new(machine_io));
    interpret(code_ref, io_ref.clone());
    io_ref
}
