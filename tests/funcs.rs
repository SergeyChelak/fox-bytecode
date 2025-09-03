use crate::common::{interpret_using_probe, interpret_with};
mod common;

#[test]
fn func_declaration_test() {
    let src = r#"
        fun areWeHavingItYet() {
          print "Yes we are!";
        }

        print areWeHavingItYet;
    "#;
    let probe = interpret_using_probe(src);
    let output = &["<closure <fn areWeHavingItYet>>"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn not_function_call_test() {
    let src = r#"
        var notAFunction = 123;
        notAFunction();
    "#;
    let probe = interpret_using_probe(src);
    assert_eq!(
        Some("Can only call functions and classes"),
        probe.borrow().top_error_message()
    );
}

#[test]
fn script_return_test() {
    let src = r#"
        return "Hello";
    "#;
    let probe = interpret_using_probe(src);
    assert_eq!(
        Some("Can't return from top-level code"),
        probe.borrow().top_error_message()
    );
}

#[test]
fn func_return_test() {
    let src = r#"
        fun sum_3(x, y, z) {
            return x + y + z;
        }

        fun mul_3(x, y, z) {
            var res = x * y * z;
            return res;
        }

        fun combined_3(x, y, z) {
            var s = sum_3(x, y, z);
            var m = mul_3(x, y, z);
            return m / s;
        }

        print "Result: " + combined_3(3, 4, 5);
        print "OK";
    "#;
    let probe = interpret_using_probe(src);
    let output = &["Result: 5", "OK"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn fibonacci_recursion_test() {
    let src = r#"
        fun fib(n) {
          if (n <= 1) return n;
          return fib(n - 2) + fib(n - 1);
        }

        for (var i = 0; i < 20; i = i + 1) {
          print fib(i);
        }

        print "OK";
    "#;
    let probe = interpret_using_probe(src);
    let output = &[
        "0", "1", "1", "2", "3", "5", "8", "13", "21", "34", "55", "89", "144", "233", "377",
        "610", "987", "1597", "2584", "4181", "OK",
    ];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn func_implicit_return_test() {
    let src = r#"
        fun noReturn() {
          print "Do stuff";
          // implicit return nil here
        }

        print noReturn();
        print "OK";
    "#;
    let probe = interpret_using_probe(src);
    let output = &["Do stuff", "nil", "OK"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

#[test]
fn stacktrace_test() {
    let src = r#"
        fun a() { b(); }
        fun b() { c(); }
        fun c() {
          c("too", "many");
        }

        a();
    "#;
    let probe = interpret_using_probe(src);
    assert_eq!(
        Some("Expected 0 arguments but got 2"),
        probe.borrow().top_error_message()
    );

    let stack_trace = probe.borrow().stack_trace_text();
    assert_eq!(
        Some("[line 5] in c\n[line 3] in b\n[line 2] in a\n[line 8] in script"),
        stack_trace.as_deref()
    );
}

#[test]
fn native_function_call_test() {
    let src = r#"
        var val = sum(1, 2, 3, 4);
        print val;
        print "OK";
    "#;
    let probe = interpret_with(src, native_funcs::Provider);
    let output = &["10", "OK"];
    assert_eq!(None, probe.borrow().top_error_message());
    probe.borrow().assert_output_match(output);
}

mod native_funcs {
    use fox_bytecode::Value;

    pub struct Provider;

    impl fox_bytecode::NativeFunctionsProvider for Provider {
        fn get_functions(&self) -> Vec<(String, fox_bytecode::NativeFn)> {
            vec![("sum".to_string(), sum)]
        }
    }

    fn sum(args: &[Value]) -> Value {
        let mut acc = 0.0;
        for x in args {
            let Some(num) = x.as_number() else {
                return Value::Nil;
            };
            acc += num;
        }
        Value::Number(acc)
    }
}
