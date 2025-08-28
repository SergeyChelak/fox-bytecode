use std::{cell::RefCell, rc::Rc};

// IO
pub fn file_to_chars<T: AsRef<str>>(path: T) -> std::io::Result<Vec<char>> {
    let p = path.as_ref();
    let data = std::fs::read_to_string(p)?;
    let code = data.chars().collect::<Vec<_>>();
    Ok(code)
}

// jump calculations
pub fn word_to_bytes(jump: usize) -> (u8, u8) {
    let first = ((jump >> 8) & 0xff) as u8;
    let second = (jump & 0xff) as u8;
    (first, second)
}

pub fn bytes_to_word(first: u8, second: u8) -> usize {
    ((first as usize) << 8) | (second as usize)
}

//
pub type Shared<T> = Rc<RefCell<T>>;

pub fn shared<T>(value: T) -> Shared<T> {
    Rc::new(RefCell::new(value))
}
