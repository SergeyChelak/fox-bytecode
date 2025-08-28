use std::io::Write;

use crate::{NativeFn, NativeFunctionsProvider, Value};

pub struct ProductionNativeFunctions;

impl NativeFunctionsProvider for ProductionNativeFunctions {
    fn get_functions(&self) -> Vec<(String, NativeFn)> {
        vec![
            ("write".to_string(), native_write),
            ("writeln".to_string(), native_writeln),
            ("readln".to_string(), native_read_line),
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

/// args are prompt
fn native_read_line(args: &[Value]) -> Value {
    native_write(args);
    if std::io::stdout().flush().is_err() {
        return Value::Nil;
    }

    // Wait for user input
    let stdin = std::io::stdin();
    let mut input = String::new();
    if stdin.read_line(&mut input).is_err() {
        return Value::Nil;
    }
    // don't include eof character, maybe windows will produce \r\n
    let len = input.len();
    Value::text_from_str(&input[0..len - 1])
}
