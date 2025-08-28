use crate::{NativeFn, Value};

pub trait NativeFunctionsProvider {
    fn get_functions(&self) -> Vec<(String, NativeFn)>;
}

pub struct ProductionNativeFunctions;

impl NativeFunctionsProvider for ProductionNativeFunctions {
    fn get_functions(&self) -> Vec<(String, NativeFn)> {
        vec![
            ("write".to_string(), native_write),
            ("writeln".to_string(), native_writeln),
        ]
    }
}

fn native_write(args: &[Value]) -> Value {
    args.iter().for_each(|x| print!("{x}"));
    Value::Nil
}

fn native_writeln(args: &[Value]) -> Value {
    native_write(args);
    println!();
    Value::Nil
}
